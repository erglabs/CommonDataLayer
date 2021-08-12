use std::sync::Arc;

use cache::DynamicCache;
use metrics_utils as metrics;
use schema::SchemaMetadataSupplier;
use settings_utils::{apps::query_router::QueryRouterSettings, load_settings};
use uuid::Uuid;
use warp::Filter;

pub mod error;
pub mod handler;
pub mod schema;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    misc_utils::set_aborting_panic_hook();

    let settings: QueryRouterSettings = load_settings()?;
    tracing_utils::init(
        settings.log.rust_log.as_str(),
        settings.monitoring.otel_service_name.as_str(),
    )?;

    tracing::debug!(?settings, "application environment");

    metrics::serve(&settings.monitoring);

    let schema_registry_cache = Arc::new(DynamicCache::new(
        settings.cache_capacity,
        SchemaMetadataSupplier::new(settings.services.schema_registry_url),
    ));

    let cache_filter = warp::any().map(move || schema_registry_cache.clone());

    let routing_table = Arc::new(settings.repositories);

    let routing_filter = warp::any().map(move || routing_table.clone());

    let schema_id_filter = warp::header::header::<Uuid>("SCHEMA_ID");
    let repository_id_filter = warp::header::optional::<String>("REPOSITORY_ID");
    let body_filter = warp::body::content_length_limit(1024 * 32).and(warp::body::json());

    let single_route = warp::path!("single" / Uuid)
        .and(schema_id_filter)
        .and(repository_id_filter)
        .and(cache_filter.clone())
        .and(routing_filter.clone())
        .and(body_filter)
        .and_then(handler::query_single);

    let multiple_route = warp::path!("multiple" / String)
        .and(schema_id_filter)
        .and(repository_id_filter)
        .and(cache_filter.clone())
        .and(routing_filter.clone())
        .and_then(handler::query_multiple);

    let schema_route = warp::path!("schema")
        .and(schema_id_filter)
        .and(repository_id_filter)
        .and(cache_filter.clone())
        .and(routing_filter.clone())
        .and_then(handler::query_by_schema);

    let raw_route = warp::path!("raw")
        .and(schema_id_filter)
        .and(repository_id_filter)
        .and(cache_filter.clone())
        .and(body_filter)
        .and(routing_filter.clone())
        .and_then(handler::query_raw);

    let routes = warp::post()
        .and(single_route.or(raw_route))
        .or(warp::get().and(multiple_route.or(schema_route)));

    tracing_utils::http::serve(routes, ([0, 0, 0, 0], settings.input_port)).await;

    Ok(())
}
