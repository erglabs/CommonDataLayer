use materializer_ondemand::MaterializerImpl;
use rpc::materializer_ondemand::on_demand_materializer_server::OnDemandMaterializerServer;
use settings_utils::{apps::materializer_ondemand::MaterializerOndemandSettings, load_settings};
use tonic::transport::Server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    misc_utils::set_aborting_panic_hook();

    let settings: MaterializerOndemandSettings = load_settings()?;
    tracing_utils::init(
        settings.log.rust_log.as_str(),
        settings.monitoring.otel_service_name.as_str(),
    )?;

    tracing::debug!(?settings, "application environment");

    utils::status_endpoints::serve(&settings.monitoring);
    metrics_utils::serve(&settings.monitoring);

    let materializer = MaterializerImpl::new(&settings.services.object_builder_url).await?;

    utils::status_endpoints::mark_as_started();

    Server::builder()
        .trace_fn(tracing_utils::grpc::trace_fn)
        .add_service(OnDemandMaterializerServer::new(materializer))
        .serve(([0, 0, 0, 0], settings.input_port).into())
        .await?;

    Ok(())
}
