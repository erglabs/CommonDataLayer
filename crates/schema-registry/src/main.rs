use std::{
    fs::File,
    net::{Ipv4Addr, SocketAddrV4},
    path::PathBuf,
};

use anyhow::Context;
use metrics_utils as metrics;
use rpc::schema_registry::schema_registry_server::SchemaRegistryServer;
use schema_registry::rpc::SchemaRegistryImpl;
use settings_utils::{apps::schema_registry::SchemaRegistrySettings, load_settings};
use tokio::time::{sleep, Duration};
use tonic::transport::Server;
use utils::status_endpoints;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    misc_utils::set_aborting_panic_hook();

    let settings: SchemaRegistrySettings = load_settings()?;
    tracing_utils::init(
        settings.log.rust_log.as_str(),
        settings.monitoring.otel_service_name.as_str(),
    )?;

    tracing::debug!(?settings, "application environment");

    sleep(Duration::from_millis(500)).await;

    status_endpoints::serve(&settings.monitoring);
    metrics::serve(&settings.monitoring);

    let registry = SchemaRegistryImpl::new(&settings).await?;

    if let Some(export_filename) = settings.export_dir.map(export_path) {
        let exported = registry.export_all().await?;
        let file = File::create(export_filename)?;
        serde_json::to_writer_pretty(&file, &exported)?;
    }

    if let Some(import_path) = settings.import_file {
        let imported = File::open(&import_path).map_err(|err| {
            anyhow::anyhow!(
                "Could not open import file {}: {}",
                import_path.display(),
                err
            )
        })?;
        let imported = serde_json::from_reader(imported)
            .with_context(|| format!("Failed to read import file: {}", import_path.display()))?;
        registry.import_all(imported).await.map_err(|err| {
            anyhow::anyhow!(
                "Failed to import database from file {}: {}",
                import_path.display(),
                err
            )
        })?;
    }

    let addr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), settings.input_port);
    status_endpoints::mark_as_started();

    Server::builder()
        .layer(tracing_utils::grpc::TraceLayer)
        .add_service(SchemaRegistryServer::new(registry))
        .serve(addr.into())
        .await
        .context("gRPC server failed")
}

fn export_path(export_dir_path: PathBuf) -> PathBuf {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Invalid system time");

    export_dir_path.join(format!("export_{:?}.json", timestamp.as_secs()))
}
