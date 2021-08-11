use anyhow::{Context, Error};
use bb8_postgres::bb8::{Pool, PooledConnection};
use bb8_postgres::tokio_postgres::{Config, NoTls};
use bb8_postgres::{bb8, PostgresConnectionManager};
use communication_utils::consumer::ConsumerHandler;
use communication_utils::message::CommunicationMessage;
use futures::future::BoxFuture;
use futures::FutureExt;
use itertools::Itertools;
use metrics_utils::{self as metrics, counter};
use notification_utils::NotificationPublisher;
use rpc::edge_registry::edge_registry_server::EdgeRegistry;
use rpc::edge_registry::{
    AddSchemaRelation, Edge, Empty, ObjectIdQuery, ObjectRelations, RelationDetails, RelationId,
    RelationIdQuery, RelationList, RelationQuery, RelationResponse, RelationTree, SchemaId,
    SchemaRelation, TreeObject, TreeQuery, ValidateRelationQuery,
};
use serde::{Deserialize, Serialize};
use settings_utils::apps::PostgresSettings;
use std::cmp::min;
use std::convert::TryInto;
use std::str::FromStr;
use std::sync::Arc;
use std::{fmt, time};
use thiserror::Error;
use tonic::{Request, Response, Status};
use tracing::{debug, error, trace};
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Relation {relation} does not exist")]
    RelationDoesNotExist { relation: Uuid },
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddEdgesMessage {
    relation_id: Uuid,
    parent_object_id: Uuid,
    child_object_ids: Vec<Uuid>,
}

#[derive(Clone)]
pub struct EdgeRegistryImpl {
    pool: Pool<PostgresConnectionManager<NoTls>>,
    schema: String,
    notification_sender: Arc<NotificationPublisher<AddEdgesMessage>>,
}

impl EdgeRegistryImpl {
    pub async fn new(
        settings: &PostgresSettings,
        notification_sender: Arc<NotificationPublisher<AddEdgesMessage>>,
    ) -> anyhow::Result<Self> {
        let mut pg_config = Config::new();
        pg_config
            .user(&settings.username)
            .password(&settings.password)
            .host(&settings.host)
            .port(settings.port)
            .dbname(&settings.dbname);
        let manager = PostgresConnectionManager::new(pg_config, NoTls);
        let pool = bb8::Pool::builder()
            .max_size(20)
            .connection_timeout(time::Duration::from_secs(30))
            .build(manager)
            .await?;

        Ok(Self {
            pool,
            schema: settings.schema.clone(),
            notification_sender,
        })
    }

    async fn set_schema(
        &self,
        conn: &PooledConnection<'_, PostgresConnectionManager<NoTls>>,
    ) -> anyhow::Result<()> {
        conn.execute(
            format!("SET search_path TO '{}'", &self.schema).as_str(),
            &[],
        )
        .await?;

        Ok(())
    }

    async fn connect(
        &self,
    ) -> anyhow::Result<PooledConnection<'_, PostgresConnectionManager<NoTls>>> {
        let conn = self.pool.get().await?;

        self.set_schema(&conn).await?;

        Ok(conn)
    }

    #[tracing::instrument(skip(self))]
    async fn add_relation_impl(
        &self,
        relation_id: Option<Uuid>,
        parent_schema_id: Uuid,
        child_schema_id: Uuid,
    ) -> anyhow::Result<Uuid> {
        counter!("cdl.edge-registry.add-relation", 1);

        let conn = self.connect().await?;

        let relation = if let Some(relation_id) = relation_id {
            conn
                .query(
                    "INSERT INTO relations (relation_id, parent_schema_id, child_schema_id) VALUES ($1::uuid, $2::uuid, $3::uuid)",
                    &[&relation_id, &parent_schema_id, &child_schema_id]
                )
                .await?;

            relation_id
        } else {
            let row = conn
                .query(
                    "INSERT INTO relations (parent_schema_id, child_schema_id) VALUES ($1::uuid, $2::uuid) RETURNING id",
                    &[&parent_schema_id, &child_schema_id]
                )
                .await?;

            row.get(0).context("Critical error, missing rows")?.get(0)
        };

        Ok(relation)
    }

    #[tracing::instrument(skip(self))]
    async fn get_relation_impl(
        &self,
        relation_id: &Uuid,
        parent_schema_id: &Uuid,
    ) -> anyhow::Result<Option<Uuid>> {
        counter!("cdl.edge-registry.get-relation", 1);

        let conn = self.connect().await?;
        Ok(conn
            .query(
                "SELECT child_schema_id FROM relations WHERE id = $1 AND parent_schema_id = $2",
                &[&relation_id, &parent_schema_id],
            )
            .await?
            .first()
            .map(|row| row.get::<_, Uuid>(0)))
    }

    async fn get_schema_by_relation_impl(
        &self,
        relation_id: Uuid,
    ) -> anyhow::Result<Option<(Uuid, Uuid)>> {
        counter!("cdl.edge-registry.get-schema-by-relation", 1);

        let conn = self.connect().await?;
        self.get_schema_by_relation_conn(&conn, relation_id).await
    }

    async fn get_schema_by_relation_conn(
        &self,
        conn: &PooledConnection<'_, PostgresConnectionManager<NoTls>>,
        relation_id: Uuid,
    ) -> anyhow::Result<Option<(Uuid, Uuid)>> {
        Ok(conn
            .query(
                "SELECT parent_schema_id, child_schema_id FROM relations WHERE id = $1",
                &[&relation_id],
            )
            .await?
            .first()
            .map(|row| (row.get(0), row.get(1))))
    }

    #[tracing::instrument(skip(self))]
    async fn validate_relation_impl(&self, relation_id: &Uuid) -> Result<(), ValidationError> {
        counter!("cdl.edge-registry.validate-relation", 1);

        let conn = self.connect().await?;
        conn.query(
            "SELECT child_schema_id FROM relations WHERE id = $1",
            &[&relation_id],
        )
        .await
        .map_err(anyhow::Error::from)?
        .first()
        .map(|_| ())
        .ok_or(ValidationError::RelationDoesNotExist {
            relation: *relation_id,
        })
    }

    #[tracing::instrument(skip(self))]
    async fn get_schema_relations_impl(
        &self,
        schema_id: &Uuid,
    ) -> anyhow::Result<impl Iterator<Item = (Uuid, Uuid)>> {
        counter!("cdl.edge-registry.get-schema-relations", 1);

        let conn = self.connect().await?;
        Ok(conn
            .query(
                "SELECT id, child_schema_id FROM relations WHERE parent_schema_id = ($1::uuid)",
                &[&schema_id],
            )
            .await?
            .into_iter()
            .map(|row| (row.get(0), row.get(1))))
    }

    #[tracing::instrument(skip(self))]
    async fn list_relations_impl(
        &self,
    ) -> anyhow::Result<impl Iterator<Item = (Uuid, Uuid, Uuid)>> {
        counter!("cdl.edge-registry.list-relations", 1);
        let conn = self.connect().await?;

        Ok(conn
            .query(
                "SELECT id, parent_schema_id, child_schema_id FROM relations",
                &[],
            )
            .await?
            .into_iter()
            .map(|row| (row.get(0), row.get(1), row.get(2))))
    }

    #[tracing::instrument(skip(self, relations))]
    async fn add_edges_impl(
        &self,
        relations: impl IntoIterator<Item = AddEdgesMessage>,
    ) -> anyhow::Result<()> {
        counter!("cdl.edge-registry.add-edges", 1);
        let conn = self.connect().await?;

        for relation in relations {
            trace!(
                "Adding {} edges in `{}`",
                relation.child_object_ids.len(),
                relation.relation_id
            );
            Arc::clone(&self.notification_sender)
                .and_message_body(&relation)
                .notify("success")
                .await?;
            for child_object_id in relation.child_object_ids {
                conn
                    .query(
                        "INSERT INTO edges (relation_id, parent_object_id, child_object_id) VALUES ($1, $2, $3)",
                        &[&relation.relation_id, &relation.parent_object_id, &child_object_id],
                    )
                    .await?;
            }
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn get_edge_impl(
        &self,
        relation_id: Uuid,
        parent_object_id: Uuid,
    ) -> anyhow::Result<impl Iterator<Item = Uuid>> {
        counter!("cdl.edge-registry.get-edge", 1);
        let conn = self.connect().await?;

        self.get_edge_with_conn(&conn, relation_id, parent_object_id)
            .await
    }

    #[tracing::instrument(skip(self, conn))]
    async fn get_edge_with_conn(
        &self,
        conn: &PooledConnection<'_, PostgresConnectionManager<NoTls>>,
        relation_id: Uuid,
        parent_object_id: Uuid,
    ) -> anyhow::Result<impl Iterator<Item = Uuid>> {
        Ok(conn
            .query(
                "SELECT child_object_id FROM edges WHERE relation_id = $1 AND parent_object_id = $2",
                &[&relation_id, &parent_object_id],
            )
            .await?
            .into_iter()
            .map(|row| row.get(0))
        )
    }

    #[tracing::instrument(skip(self))]
    async fn get_edges_impl(&self, object_id: Uuid) -> anyhow::Result<Vec<Edge>> {
        counter!("cdl.edge-registry.get-edges", 1);
        let conn = self.connect().await?;
        Ok(conn
            .query(
                "SELECT relation_id, child_object_id FROM edges WHERE parent_object_id = $1 ORDER BY relation_id",
                &[&object_id],
            )
            .await?
            .into_iter()
            .map(|row| (row.get::<_,Uuid>(0), row.get::<_,Uuid>(1)))
            .group_by(|(relation_id, _child_object_id)| *relation_id)
            .into_iter()
            .map(move |(relation_id, children)| Edge {
                relation_id: relation_id.to_string(),
                parent_object_id: object_id.to_string(),
                child_object_ids: children.map(|child| child.1.to_string()).collect(),
            }).collect())
    }

    fn resolve_tree_recursive<'a, F, S, R>(
        &'a self,
        conn: &'a PooledConnection<PostgresConnectionManager<NoTls>>,
        relation_id: R,
        filter_ids: F,
        relations: &'a [TreeQuery],
    ) -> BoxFuture<'_, anyhow::Result<RelationTree>>
    where
        F: IntoIterator<Item = S> + Send + Sync + 'a,
        <F as IntoIterator>::IntoIter: Send + Sync,
        S: AsRef<str> + Send + Sync,
        R: AsRef<str> + Send + Sync + 'a,
    {
        async move {
            let relation_id = relation_id.as_ref();

            let mut filter_ids = filter_ids.into_iter().peekable();

            if filter_ids.peek().is_none() {
                self.resolve_tree_for_ids(
                    conn,
                    relation_id,
                    relations,
                    conn.query(
                        "SELECT DISTINCT parent_object_id FROM edges WHERE relation_id = $1",
                        &[&relation_id.parse::<Uuid>()?],
                    )
                    .await?
                    .into_iter()
                    .map(|row| row.get::<_, Uuid>(0).to_string()),
                )
                .await
            } else {
                self.resolve_tree_for_ids(conn, relation_id, relations, filter_ids)
                    .await
            }
        }
        .boxed()
    }

    fn resolve_tree_for_ids<'a, F, S, R>(
        &'a self,
        conn: &'a PooledConnection<'_, PostgresConnectionManager<NoTls>>,
        relation_id: R,
        relations: &'a [TreeQuery],
        filter_ids: F,
    ) -> BoxFuture<'_, anyhow::Result<RelationTree, Error>>
    where
        F: IntoIterator<Item = S> + Send + Sync + 'a,
        <F as IntoIterator>::IntoIter: Send + Sync,
        S: AsRef<str> + Send + Sync,
        R: AsRef<str> + Send + Sync + 'a,
    {
        async move {
            let mut objects = Vec::new();
            let relation = self
                .get_schema_by_relation_conn(conn, relation_id.as_ref().parse()?)
                .await?
                .with_context(|| format!("Relation {} does not exist", relation_id.as_ref()))?;
            for object_id in filter_ids {
                let children = self
                    .get_edge_with_conn(
                        conn,
                        relation_id.as_ref().parse()?,
                        object_id.as_ref().parse()?,
                    )
                    .await?
                    .map(|uuid| uuid.to_string())
                    .collect::<Vec<_>>();

                if !children.is_empty() {
                    let mut subtrees = Vec::with_capacity(relations.len());
                    for relation in relations.iter() {
                        let subtree = if relation.filter_ids.is_empty() {
                            self.resolve_tree_recursive(
                                conn,
                                &relation.relation_id,
                                &children,
                                &relation.relations,
                            )
                            .await?
                        } else {
                            let object_ids = intersect(&children, &relation.filter_ids);
                            if !object_ids.is_empty() {
                                self.resolve_tree_recursive(
                                    conn,
                                    &relation.relation_id,
                                    object_ids.iter(),
                                    &relation.relations,
                                )
                                .await?
                            } else {
                                RelationTree { objects: vec![] }
                            }
                        };
                        subtrees.push(subtree);
                    }
                    objects.push(TreeObject {
                        object_id: object_id.as_ref().to_string(),
                        relation_id: relation_id.as_ref().to_string(),
                        children,
                        subtrees,
                        relation: SchemaRelation {
                            parent_schema_id: relation.0.to_string(),
                            child_schema_id: relation.1.to_string(),
                        },
                    });
                }
            }

            Ok(RelationTree { objects })
        }
        .boxed()
    }
}

fn intersect<T: PartialEq + Clone>(left: &[T], right: &[T]) -> Vec<T> {
    let mut result = Vec::with_capacity(min(left.len(), right.len()));

    for item in left {
        if right.contains(item) {
            result.push(item.clone())
        }
    }

    result
}

#[tonic::async_trait]
impl EdgeRegistry for EdgeRegistryImpl {
    async fn add_relation(
        &self,
        request: Request<AddSchemaRelation>,
    ) -> Result<Response<RelationId>, Status> {
        let request = request.into_inner();

        trace!(
            "Received `add_relation` message with parent_id `{}` and child_id `{}` and relation_id `{:?}`",
            request.parent_schema_id,
            request.child_schema_id,
            request.relation_id,
        );

        let relation_id = request
            .relation_id
            .as_ref()
            .map(|relation_id| {
                Uuid::from_str(relation_id).map_err(|_| Status::invalid_argument("relation_id"))
            })
            .transpose()?;
        let parent_schema_id = Uuid::from_str(&request.parent_schema_id)
            .map_err(|_| Status::invalid_argument("parent_schema_id"))?;
        let child_schema_id = Uuid::from_str(&request.child_schema_id)
            .map_err(|_| Status::invalid_argument("child_schema_id"))?;

        let relation_id = self
            .add_relation_impl(relation_id, parent_schema_id, child_schema_id)
            .await
            .map_err(|err| db_communication_error("add_relation", err))?;

        Ok(Response::new(RelationId {
            relation_id: relation_id.to_string(),
        }))
    }

    async fn get_relation(
        &self,
        request: Request<RelationQuery>,
    ) -> Result<Response<RelationResponse>, Status> {
        let request = request.into_inner();

        trace!(
            "Received `get_relation` message with relation_id `{}` and parent_id `{}`",
            request.relation_id,
            request.parent_schema_id
        );

        let relation_id = Uuid::from_str(&request.relation_id)
            .map_err(|_| Status::invalid_argument("relation_id"))?;
        let parent_schema_id = Uuid::from_str(&request.parent_schema_id)
            .map_err(|_| Status::invalid_argument("parent_schema_id"))?;

        let child_schema_id = self
            .get_relation_impl(&relation_id, &parent_schema_id)
            .await
            .map_err(|err| db_communication_error("get_relation", err))?;

        Ok(Response::new(RelationResponse {
            child_schema_id: child_schema_id.map(|id| id.to_string()),
        }))
    }

    async fn get_schema_by_relation(
        &self,
        request: Request<RelationId>,
    ) -> Result<Response<SchemaRelation>, Status> {
        let request = request.into_inner();

        trace!(
            "Received `get_schema_by_relation` message with relation_id `{}`",
            request.relation_id
        );

        let relation_id = Uuid::from_str(&request.relation_id)
            .map_err(|_| Status::invalid_argument("relation_id"))?;

        let relation = self
            .get_schema_by_relation_impl(relation_id)
            .await
            .map_err(|err| db_communication_error("get_schema_by_relation", err))?;

        if let Some((parent, child)) = relation {
            Ok(Response::new(SchemaRelation {
                parent_schema_id: parent.to_string(),
                child_schema_id: child.to_string(),
            }))
        } else {
            Err(Status::not_found("Specified relation doesn't exist"))
        }
    }

    async fn validate_relation(
        &self,
        request: Request<ValidateRelationQuery>,
    ) -> Result<Response<Empty>, Status> {
        let request = request.into_inner();

        let relation_id = Uuid::from_str(&request.relation_id)
            .map_err(|_| Status::invalid_argument("relation_id"))?;

        match self.validate_relation_impl(&relation_id).await {
            Ok(_) => Ok(Response::new(Empty {})),
            Err(ValidationError::Unexpected(e)) => {
                Err(db_communication_error("validate_relation", e))
            }
            Err(e) => Err(Status::invalid_argument(format!("{}", e))),
        }
    }

    async fn get_schema_relations(
        &self,
        request: Request<SchemaId>,
    ) -> Result<Response<RelationList>, Status> {
        let request = request.into_inner();

        trace!(
            "Received `get_schema_relations` message with schema_id `{}`",
            request.schema_id
        );

        let schema_id = Uuid::from_str(&request.schema_id)
            .map_err(|_| Status::invalid_argument("schema_id"))?;

        let rows = self
            .get_schema_relations_impl(&schema_id)
            .await
            .map_err(|err| db_communication_error("get_schema_relations", err))?;

        Ok(Response::new(RelationList {
            items: rows
                .map(|(relation_id, child_schema_id)| RelationDetails {
                    relation_id: relation_id.to_string(),
                    parent_schema_id: request.schema_id.clone(),
                    child_schema_id: child_schema_id.to_string(),
                })
                .collect(),
        }))
    }

    async fn list_relations(&self, _: Request<Empty>) -> Result<Response<RelationList>, Status> {
        trace!("Received `list_relations` message");

        let rows = self
            .list_relations_impl()
            .await
            .map_err(|err| db_communication_error("list_relations", err))?;

        Ok(Response::new(RelationList {
            items: rows
                .map(
                    |(relation_id, parent_schema_id, child_schema_id)| RelationDetails {
                        relation_id: relation_id.to_string(),
                        parent_schema_id: parent_schema_id.to_string(),
                        child_schema_id: child_schema_id.to_string(),
                    },
                )
                .collect(),
        }))
    }

    async fn add_edges(
        &self,
        request: Request<ObjectRelations>,
    ) -> Result<Response<Empty>, Status> {
        counter!("cdl.edge-registry.add-edges.grpc", 1);
        let request = request.into_inner();

        let edges = request
            .relations
            .into_iter()
            .map(|relation| {
                Ok(AddEdgesMessage {
                    relation_id: Uuid::from_str(&relation.relation_id)
                        .with_context(|| format!("relation_id {}", relation.relation_id))?,
                    parent_object_id: Uuid::from_str(&relation.parent_object_id).with_context(
                        || format!("parent_object_id {}", relation.parent_object_id),
                    )?,
                    child_object_ids: relation
                        .child_object_ids
                        .into_iter()
                        .map(|child_object_id| {
                            Uuid::from_str(&child_object_id)
                                .with_context(|| format!("child_object_id {}", child_object_id))
                        })
                        .collect::<anyhow::Result<Vec<Uuid>>>()?,
                })
            })
            .collect::<anyhow::Result<Vec<AddEdgesMessage>>>()
            .map_err(|err| {
                debug!("Failed deserializing `add_edges` query. {:?}", err);
                Status::invalid_argument(
                    "Failed to deserialize query. Check if all uuids are in correct format.",
                )
            })?;

        counter!(
            "cdl.edge-registry.add-edges.count",
            edges.len().try_into().unwrap()
        );
        trace!(
            "Received `add_edges` message with {} relations",
            edges.len()
        );

        self.add_edges_impl(edges)
            .await
            .map_err(|err| db_communication_error("add_edges", err))?;

        Ok(Response::new(Empty {}))
    }

    async fn get_edge(&self, request: Request<RelationIdQuery>) -> Result<Response<Edge>, Status> {
        let request = request.into_inner();

        trace!(
            "Received `get_edge` message with relation_id `{}` and parent_id `{}`",
            request.relation_id,
            request.parent_object_id
        );

        let relation_id = Uuid::from_str(&request.relation_id)
            .map_err(|_| Status::invalid_argument("relation_id"))?;
        let parent_object_id = Uuid::from_str(&request.parent_object_id)
            .map_err(|_| Status::invalid_argument("parent_object_id"))?;

        let rows = self
            .get_edge_impl(relation_id, parent_object_id)
            .await
            .map_err(|err| db_communication_error("get_edge", err))?;

        Ok(Response::new(Edge {
            relation_id: request.relation_id.to_string(),
            parent_object_id: request.parent_object_id.to_string(),
            child_object_ids: rows.map(|uuid| uuid.to_string()).collect(),
        }))
    }

    async fn get_edges(
        &self,
        request: Request<ObjectIdQuery>,
    ) -> Result<Response<ObjectRelations>, Status> {
        let request = request.into_inner();

        trace!(
            "Received `get_edge` message with object_id `{}`",
            request.object_id
        );

        let object_id = Uuid::from_str(&request.object_id)
            .map_err(|_| Status::invalid_argument("object_id"))?;

        let rows = self
            .get_edges_impl(object_id)
            .await
            .map_err(|err| db_communication_error("get_edges", err))?;

        Ok(Response::new(ObjectRelations { relations: rows }))
    }

    #[tracing::instrument(skip(self))]
    async fn heartbeat(&self, _request: Request<Empty>) -> Result<Response<Empty>, Status> {
        //empty
        Ok(Response::new(Empty {}))
    }

    #[tracing::instrument(skip(self))]
    async fn resolve_tree(
        &self,
        request: Request<TreeQuery>,
    ) -> Result<Response<RelationTree>, Status> {
        let request = request.into_inner();

        trace!(
            "Received `resolve_tree` message with root `{}`",
            request.relation_id
        );

        let result = self
            .resolve_tree_recursive(
                &self
                    .connect()
                    .await
                    .map_err(|err| db_communication_error("resolve_tree", err))?,
                request.relation_id,
                request.filter_ids.iter(),
                &request.relations,
            )
            .await
            .map_err(|err| db_communication_error("resolve_tree", err))?;

        Ok(Response::new(result))
    }
}

#[async_trait::async_trait]
impl ConsumerHandler for EdgeRegistryImpl {
    #[tracing::instrument(skip(self, msg))]
    async fn handle<'a>(&'a mut self, msg: &'a dyn CommunicationMessage) -> anyhow::Result<()> {
        counter!("cdl.edge-registry.add-edges.mq", 1);

        let edges: Vec<AddEdgesMessage> = serde_json::from_str(msg.payload()?)?;

        counter!(
            "cdl.edge-registry.add-edges.count",
            edges.len().try_into().unwrap()
        );

        trace!("Consuming `add_edges` with {} relations", edges.len());

        self.add_edges_impl(edges).await?;

        Ok(())
    }
}

fn db_communication_error(text: &str, err: impl fmt::Debug) -> Status {
    error!(
        "`{}` query failed on communication with database: `{:?}`",
        text, err
    );
    Status::internal("Query failed on communication with database")
}
