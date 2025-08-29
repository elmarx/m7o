use k8s_openapi::api::core::v1::ConfigMap;
use kube::{Api, Client, Error, api::ListParams};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();
    let client = Client::try_default().await?;

    let cms = Api::<ConfigMap>::default_namespaced(client);

    let params = ListParams::default().labels("tag=elmar");
    // let params = ListParams::default();

    let cms = cms.list(&params).await?;

    info!("{:#?}", cms);

    Ok(())
}
