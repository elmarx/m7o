use std::sync::Arc;
use std::time::Duration;

use futures::{StreamExt, future};
use k8s_openapi::api::core::v1::Pod;
use kube::ResourceExt;
use kube::runtime::Controller;
use kube::{Api, Client, Error, runtime::controller::Action};
use tracing::info;

#[derive(thiserror::Error, Debug)]
pub enum M7oError {}

pub type Result<T, E = M7oError> = std::result::Result<T, E>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let client = Client::try_default().await?;
    let pods = Api::<Pod>::all(client);

    Controller::new(pods.clone(), Default::default())
        .run(reconcile, error_policy, Arc::new(()))
        .for_each(|_| future::ready(()))
        .await;

    Ok(())
}

fn error_policy(_object: Arc<Pod>, _err: &M7oError, _ctx: Arc<()>) -> Action {
    Action::requeue(Duration::from_secs(5))
}

async fn reconcile(obj: Arc<Pod>, _ctx: Arc<()>) -> Result<Action> {
    info!("reconcile request: {}", obj.name_any());

    Ok(Action::requeue(Duration::from_secs(3600)))
}
