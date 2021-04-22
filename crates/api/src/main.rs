pub mod config;
pub mod error;
pub mod events;
pub mod schema;
pub mod types;

use std::convert::Infallible;

use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::Schema;
use async_graphql_warp::{graphql_subscription, Response};
use clap::Clap;
use warp::{http::Response as HttpResponse, hyper::header::CONTENT_TYPE, hyper::Method, Filter};

use crate::schema::context::EdgeRegistryConnectionManager;
use config::Config;
use schema::context::{MQEvents, ObjectBuilderConnectionManager, SchemaRegistryConnectionManager};
use schema::{mutation::MutationRoot, query::QueryRoot, subscription::SubscriptionRoot};

#[tokio::main]
async fn main() {
    utils::set_aborting_panic_hook();
    utils::tracing::init();

    let config = Config::parse();
    let input_port = config.input_port;

    let cors = warp::cors()
        .allow_methods(&[Method::POST, Method::GET, Method::OPTIONS])
        .allow_headers(&[CONTENT_TYPE])
        .allow_any_origin();

    let sr_pool = bb8::Pool::builder()
        .build(SchemaRegistryConnectionManager {
            address: config.schema_registry_addr.clone(),
        })
        .await
        .unwrap();

    let er_pool = bb8::Pool::builder()
        .build(EdgeRegistryConnectionManager {
            address: config.edge_registry_addr.clone(),
        })
        .await
        .unwrap();

    let ob_pool = bb8::Pool::builder()
        .build(ObjectBuilderConnectionManager {
            address: config.object_builder_addr.clone(),
        })
        .await
        .unwrap();

    let schema = Schema::build(QueryRoot, MutationRoot, SubscriptionRoot)
        .data(config)
        .data(sr_pool)
        .data(er_pool)
        .data(ob_pool)
        .data(MQEvents {
            events: Default::default(),
        })
        .finish();

    let graphql_post = async_graphql_warp::graphql(schema.clone()).and_then(
        |(schema, request): (
            Schema<QueryRoot, MutationRoot, SubscriptionRoot>,
            async_graphql::Request,
        )| async move { Ok::<_, Infallible>(Response::from(schema.execute(request).await)) },
    );

    let graphql_playground = warp::path!("graphiql").and(warp::get()).map(|| {
        HttpResponse::builder()
            .header("content-type", "text/html")
            .body(playground_source(
                GraphQLPlaygroundConfig::new("/graphql").subscription_endpoint("/subscriptions"),
            ))
    });

    let routes = warp::path!("subscriptions")
        .and(graphql_subscription(schema))
        .or(graphql_playground)
        .or(warp::path!("graphql").and(graphql_post).with(cors));
    warp::serve(routes).run(([0, 0, 0, 0], input_port)).await;
}
