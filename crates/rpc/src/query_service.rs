pub use crate::codegen::query_service::*;
use crate::error::ClientError;
use futures_util::{Stream, TryStreamExt};
use query_service_client::QueryServiceClient;
use std::pin::Pin;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::{Channel, Endpoint, Error};
use tracing_utils::grpc::InterceptorType;

pub type ObjectStream<Error = ClientError> =
    Pin<Box<dyn Stream<Item = Result<Object, Error>> + Send + Sync + 'static>>;

pub async fn connect(
    addr: String,
) -> Result<
    QueryServiceClient<InterceptedService<Channel, &'static dyn InterceptorType>>,
    ClientError,
> {
    connect_inner(addr)
        .await
        .map_err(|err| ClientError::ConnectionError { source: err })
}

async fn connect_inner(
    addr: String,
) -> Result<QueryServiceClient<InterceptedService<Channel, &'static dyn InterceptorType>>, Error> {
    let conn = Endpoint::new(addr)?.connect().await?;

    Ok(QueryServiceClient::with_interceptor(
        conn,
        &tracing_utils::grpc::interceptor,
    ))
}

pub async fn query_multiple(
    object_ids: Vec<String>,
    addr: String,
) -> Result<ObjectStream, ClientError> {
    let mut conn = connect(addr).await?;
    let stream = conn
        .query_multiple(ObjectIds { object_ids })
        .await
        .map_err(|err| ClientError::QueryError { source: err })?;

    let stream = Box::pin(
        stream
            .into_inner()
            .map_err(|err| ClientError::QueryError { source: err }),
    );

    Ok(stream)
}

pub async fn query_by_schema(schema_id: String, addr: String) -> Result<ObjectStream, ClientError> {
    let mut conn = connect(addr).await?;
    let stream = conn
        .query_by_schema(SchemaId { schema_id })
        .await
        .map_err(|err| ClientError::QueryError { source: err })?;

    let stream = Box::pin(
        stream
            .into_inner()
            .map_err(|err| ClientError::QueryError { source: err }),
    );

    Ok(stream)
}

pub async fn query_raw(raw_statement: String, addr: String) -> Result<Vec<u8>, ClientError> {
    let mut conn = connect(addr).await?;
    let response = conn
        .query_raw(RawStatement { raw_statement })
        .await
        .map_err(|err| ClientError::QueryError { source: err })?;

    Ok(response.into_inner().value_bytes)
}
