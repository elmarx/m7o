use k8s_openapi::api::core::v1::ConfigMap;
use kube::{Api, Client, Error, api::ListParams};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client = Client::try_default().await?;

    let cms = Api::<ConfigMap>::default_namespaced(client);

    let params = ListParams::default().labels("tag=elmar");

    let cms = cms.list(&params).await?;

    println!("{:#?}", cms);

    Ok(())
}
