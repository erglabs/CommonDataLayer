use ::utils::notification::NotificationPublisher;
use async_trait::async_trait;
use bb8::Pool;
use cdl_dto::{edges::RelationTree, materialization};
use communication_utils::{consumer::ConsumerHandler, message::CommunicationMessage};
use futures::{future::ready, Stream, StreamExt, TryStreamExt};
use itertools::Itertools;
use metrics_utils::{self as metrics, counter};
use row_builder::RowBuilder;
use rpc::common::{Object, RowDefinition as RpcRowDefinition};
use rpc::materializer_general::{MaterializedView as RpcMaterializedView, Options};
use rpc::object_builder::{object_builder_server::ObjectBuilder, Empty, View};
use rpc::schema_registry::types::SchemaType;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::{collections::HashMap, convert::TryInto, pin::Pin};
use tracing_futures::Instrument;
use uuid::Uuid;

use crate::{buffer_stream::ObjectBufferedStream, view_plan::ViewPlan};
use object_id_pair::ObjectIdPair;
use rpc::edge_registry::{EdgeRegistryConnectionManager, EdgeRegistryPool};
use rpc::schema_registry::{SchemaRegistryConnectionManager, SchemaRegistryPool};

pub mod settings;

mod utils;

mod buffer_stream;
mod view_plan;

mod object_id_pair;
mod row_builder;
mod sources;

#[derive(Clone)]
pub struct ObjectBuilderImpl {
    sr_pool: SchemaRegistryPool,
    er_pool: EdgeRegistryPool,
    report_sender: NotificationPublisher<materialization::Request>,
    chunk_capacity: usize,
}

type DynStream<T, E = anyhow::Error> =
    Pin<Box<dyn Stream<Item = Result<T, E>> + Send + Sync + 'static>>;

type ObjectStream = DynStream<(Uuid, Value)>;
type SchemaObjectStream = DynStream<(ObjectIdPair, Value)>;
type RowStream = DynStream<RowDefinition>;
/// This type is not Sync because async_trait futures are not Sync.
/// We require Sync in DynStream because it is requirement of tonic.
type MaterializedChunksStream =
    Pin<Box<dyn Stream<Item = anyhow::Result<MaterializedView>> + Send + 'static>>;
type MaterializeStream = DynStream<RpcRowDefinition, tonic::Status>;

#[derive(Serialize, Debug)]
#[serde(rename_all = "snake_case")]
struct MaterializedView {
    view_id: Uuid,
    options: Value,
    rows: Vec<RowDefinition>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct RowDefinition {
    objects: HashSet<ObjectIdPair>,
    fields: HashMap<String, Value>,
}

impl ObjectBuilderImpl {
    pub async fn new(settings: &settings::Settings) -> anyhow::Result<Self> {
        let sr_pool = Pool::builder()
            .build(SchemaRegistryConnectionManager {
                address: settings.services.schema_registry_url.to_string(),
            })
            .await
            .unwrap();

        let er_pool = Pool::builder()
            .build(EdgeRegistryConnectionManager {
                address: settings.services.edge_registry_url.to_string(),
            })
            .await
            .unwrap();

        let report_sender = settings
            .notifications
            .publisher(
                || async { settings.publisher().await },
                settings.communication_method.to_string(),
                "ObjectBuilder",
            )
            .await?;

        Ok(Self {
            sr_pool,
            er_pool,
            report_sender,
            chunk_capacity: settings.chunk_capacity,
        })
    }
}

impl TryInto<RpcRowDefinition> for RowDefinition {
    type Error = serde_json::Error;

    fn try_into(self) -> Result<RpcRowDefinition, Self::Error> {
        let fields = self
            .fields
            .into_iter()
            .map(|(key, value)| Ok((key, serde_json::to_string(&value)?)))
            .collect::<serde_json::Result<_>>()?;
        Ok(RpcRowDefinition {
            objects: self
                .objects
                .into_iter()
                .map(|object| Object {
                    object_id: object.object_id.to_string(),
                    schema_id: object.schema_id.to_string(),
                })
                .collect(),
            fields,
        })
    }
}

impl TryInto<RpcMaterializedView> for MaterializedView {
    type Error = serde_json::Error;

    fn try_into(self) -> Result<RpcMaterializedView, Self::Error> {
        let rows = self
            .rows
            .into_iter()
            .map(|row| row.try_into())
            .collect::<serde_json::Result<_>>()?;

        Ok(RpcMaterializedView {
            view_id: self.view_id.to_string(),
            options: Options {
                options: serde_json::to_string(&self.options)?,
            },
            rows,
        })
    }
}

#[async_trait]
impl ConsumerHandler for ObjectBuilderImpl {
    #[tracing::instrument(skip(self, msg))]
    async fn handle<'a>(&'a mut self, msg: &'a dyn CommunicationMessage) -> anyhow::Result<()> {
        let payload = msg.payload()?;
        tracing::debug!(?payload, "Handle MQ message");
        counter!("cdl.object-builder.build-object.mq", 1);
        let request: materialization::Request = serde_json::from_str(payload)?;
        let view_id = request.view_id;

        let view = self.get_view(view_id);
        let chunks = self.build_materialized_chunks(request);

        let (view, mut chunks) = futures::try_join!(view, chunks)?;

        let mut materializer =
            rpc::materializer_general::connect(view.materializer_address).await?;

        while let Some(chunk) = chunks.try_next().await? {
            let rpc_output: RpcMaterializedView = chunk.try_into()?;
            materializer.upsert_view(rpc_output).await?;
        }

        Ok(())
    }
}

#[tonic::async_trait]
impl ObjectBuilder for ObjectBuilderImpl {
    type MaterializeStream = MaterializeStream;

    #[tracing::instrument(skip(self))]
    async fn materialize(
        &self,
        request: tonic::Request<View>,
    ) -> Result<tonic::Response<Self::MaterializeStream>, tonic::Status> {
        let view: View = request.into_inner();

        let request: materialization::Request = view
            .try_into()
            .map_err(|_| tonic::Status::invalid_argument("view"))?;

        let rows = self
            .build_rows(request)
            .await
            .map_err(|err| tonic::Status::internal(format!("{}", err)))?;

        let stream = rows
            .map_err(|err| {
                tracing::error!("Could not build materialized row: {:?}", err);
                tonic::Status::internal("Could not build materialized row")
            })
            .and_then(|row| async move {
                row.try_into().map_err(|err| {
                    tracing::error!("Could not serialize materialized row: {:?}", err);
                    tonic::Status::internal("Could not serialize materialized row")
                })
            })
            .in_current_span();

        let stream = Box::pin(stream);

        Ok(tonic::Response::new(stream))
    }

    #[tracing::instrument(skip(self))]
    async fn heartbeat(
        &self,
        _request: tonic::Request<Empty>,
    ) -> Result<tonic::Response<Empty>, tonic::Status> {
        //empty
        Ok(tonic::Response::new(Empty {}))
    }
}

impl ObjectBuilderImpl {
    #[tracing::instrument(skip(self))]
    async fn build_materialized_chunks(
        &self,
        request: materialization::Request,
    ) -> anyhow::Result<MaterializedChunksStream> {
        tracing::debug!(?request, "Handling");

        let view_id = request.view_id;

        // TODO: We download view twice.
        // First time here, second time in `build_rows()`
        // Either use some kind of cache or extract `self.get_view() higher`
        let view = self.get_view(view_id).await?;
        tracing::debug!(?view, "View");

        let report_sender = self.report_sender.clone().and_message_body(&request);

        let rows = self.build_rows(request).await?;
        let rows = rows.chunks(self.chunk_capacity);

        let chunks = rows
            .map(Ok)
            .and_then(move |rows: Vec<anyhow::Result<RowDefinition>>| {
                let options = view.materializer_options.clone();
                let report_sender = report_sender.clone();
                async move {
                    let (rows, errors): (Vec<RowDefinition>, Vec<String>) = rows
                        .into_iter()
                        .map(|r| r.map_err(|e| format!("Could not build a row: {:?}", e)))
                        .partition_result();

                    for error in errors.into_iter() {
                        tracing::error!("Could not materialize row: {}", error);
                        report_sender.clone().notify(error.as_str()).await?;
                    }

                    Ok(MaterializedView {
                        view_id,
                        options,
                        rows,
                    })
                }
            });

        let stream = Box::pin(chunks) as MaterializedChunksStream;

        Ok(stream)
    }

    #[tracing::instrument(skip(self))]
    async fn build_rows(&self, request: materialization::Request) -> anyhow::Result<RowStream> {
        let materialization::Request { view_id, schemas } = request;

        let view = self.get_view(view_id).await?;
        tracing::debug!(?view, "View");

        let object_filters = create_object_filters(&schemas);
        let edges = self.resolve_tree(&view, &object_filters).await?;

        let view_plan = ViewPlan::try_new(view, &edges)?;

        let filter = view_plan.objects_filter();
        let objects = self.get_objects(view_id, filter).await?;

        let buffered_objects = ObjectBufferedStream::new(objects, view_plan);
        let row_builder = RowBuilder::new();
        let rows = buffered_objects.try_filter_map(move |row| ready(row_builder.build(row)));

        let rows = Box::pin(rows) as RowStream;

        Ok(rows)
    }

    #[tracing::instrument(skip(self))]
    async fn get_objects(
        &self,
        view_id: Uuid,
        mut schemas: HashMap<Uuid, materialization::Schema>,
    ) -> anyhow::Result<SchemaObjectStream> {
        if schemas.is_empty() {
            let base_schema = self.get_base_schema_for_view(view_id).await?;
            let base_schema_id: Uuid = base_schema.id.parse()?;
            schemas.insert(base_schema_id, Default::default());
        }

        let mut streams = vec![];
        for (schema_id, schema) in schemas.into_iter() {
            let stream = self
                .get_objects_for_ids(schema_id, &schema.object_ids)
                .await?
                .map_ok(move |(object_id, object)| {
                    (
                        ObjectIdPair {
                            schema_id,
                            object_id,
                        },
                        object,
                    )
                });
            streams.push(stream);
        }
        let stream = futures::stream::select_all(streams);
        Ok(Box::pin(stream) as SchemaObjectStream)
    }

    async fn get_base_schema_for_view(
        &self,
        view_id: Uuid,
    ) -> anyhow::Result<rpc::schema_registry::Schema> {
        let schema = self
            .sr_pool
            .get()
            .await?
            .get_base_schema_of_view(rpc::schema_registry::Id {
                id: view_id.to_string(),
            })
            .await?
            .into_inner();
        Ok(schema)
    }

    #[tracing::instrument(skip(self))]
    async fn get_objects_for_ids(
        &self,
        schema_id: Uuid,
        object_ids: &HashSet<Uuid>,
    ) -> anyhow::Result<ObjectStream> {
        let schema_meta = self.get_schema_metadata(schema_id).await?;

        let query_address = schema_meta.query_address.clone();
        let schema_type = schema_meta.schema_type.try_into()?;

        match schema_type {
            SchemaType::DocumentStorage => {
                let values = if object_ids.is_empty() {
                    rpc::query_service::query_by_schema(schema_id.to_string(), query_address).await
                } else {
                    rpc::query_service::query_multiple(
                        object_ids.iter().map(|id| id.to_string()).collect(),
                        query_address,
                    )
                    .await
                }?;

                let stream = Box::pin(values.map_err(anyhow::Error::from).and_then(
                    |object| async move {
                        let id: Uuid = object.object_id.parse()?;
                        let payload: Value = serde_json::from_slice(&object.payload)?;
                        Ok((id, payload))
                    },
                )) as ObjectStream;

                Ok(stream)
            }

            SchemaType::Timeseries => {
                // TODO:
                anyhow::bail!("Timeseries storage is not supported yet")
            }
        }
    }

    #[tracing::instrument(skip(self))]
    async fn get_view(&self, view_id: Uuid) -> anyhow::Result<cdl_dto::materialization::FullView> {
        let view = self
            .sr_pool
            .get()
            .await?
            .get_view(rpc::schema_registry::Id {
                id: view_id.to_string(),
            })
            .await?
            .into_inner();

        let view = cdl_dto::TryFromRpc::try_from_rpc(view)?;
        Ok(view)
    }

    #[tracing::instrument(skip(self))]
    async fn get_schema_metadata(
        &self,
        schema_id: Uuid,
    ) -> anyhow::Result<rpc::schema_registry::SchemaMetadata> {
        let schema = self
            .sr_pool
            .get()
            .await?
            .get_schema_metadata(rpc::schema_registry::Id {
                id: schema_id.to_string(),
            })
            .await?
            .into_inner();
        Ok(schema)
    }

    #[tracing::instrument(skip(self))]
    async fn resolve_tree(
        &self,
        view: &cdl_dto::materialization::FullView,
        object_filters: &[Uuid],
    ) -> anyhow::Result<Vec<RelationTree>> {
        let mut relations = Vec::default();
        for relation in view.relations.iter() {
            let request = into_resolve_tree_request(relation, object_filters);
            let tree = self
                .er_pool
                .get()
                .await?
                .resolve_tree(request)
                .await?
                .into_inner();

            let tree = RelationTree::from_rpc(tree)?;
            relations.push(tree);
        }
        Ok(relations)
    }
}

fn into_resolve_tree_request(
    relation: &cdl_dto::materialization::Relation,
    object_filters: &[Uuid],
) -> rpc::edge_registry::TreeQuery {
    rpc::edge_registry::TreeQuery {
        relation_id: relation.global_id.to_string(),
        relations: relation
            .relations
            .iter()
            .map(|r| into_resolve_tree_request(r, object_filters))
            .collect(),
        filter_ids: object_filters.iter().map(|o| o.to_string()).collect(),
    }
}

fn create_object_filters(schemas: &HashMap<Uuid, materialization::Schema>) -> Vec<Uuid> {
    schemas
        .values()
        .map(|s| s.object_ids.iter().copied())
        .flatten()
        .collect()
}
