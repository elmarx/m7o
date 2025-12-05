use std::sync::Arc;
use std::time::Duration;

use crate::v1::MqttBroker;
use futures::{StreamExt, future};
use kube::runtime::{Controller, watcher};
use kube::{Api, Client, Error, ResourceExt, runtime::controller::Action};
use tracing::info;

pub mod v1;

#[derive(thiserror::Error, Debug)]
pub enum M7oError {}

pub type Result<T, E = M7oError> = std::result::Result<T, E>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let client = Client::try_default().await?;
    let broker = Api::<MqttBroker>::all(client);

    Controller::new(broker.clone(), watcher::Config::default())
        .run(reconcile, error_policy, Arc::new(()))
        .for_each(|_| future::ready(()))
        .await;

    Ok(())
}

fn error_policy(_object: Arc<MqttBroker>, _err: &M7oError, _ctx: Arc<()>) -> Action {
    Action::requeue(Duration::from_secs(5))
}

async fn reconcile(obj: Arc<MqttBroker>, _ctx: Arc<()>) -> Result<Action> {
    info!("reconcile request: {}", obj.name_any());

    Ok(Action::requeue(Duration::from_secs(3600)))
}
