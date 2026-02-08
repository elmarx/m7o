use std::sync::Arc;
use std::time::Duration;

use crate::v1::{MqttBroker, MqttUser};
use error::M7oError;
use futures::{StreamExt, future};
use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::core::v1::{ConfigMap, Secret, Service};
use kube::api::{ListParams, PatchParams, PostParams};
use kube::runtime::reflector::ObjectRef;
use kube::runtime::{Controller, watcher};
use kube::{Api, Client, Error, ResourceExt, runtime::controller::Action};
use tracing::{error, info};

mod broker;
mod cm_ext;
mod error;
mod labels;
mod plan;
mod reconcile;
mod user;
mod util;
pub mod v1;

pub type Result<T, E = M7oError> = std::result::Result<T, E>;

pub const MANAGER: &str = "m7o.athmer.cloud";

const MOSQUITTO_VERSION: &str = "2.1.0-alpine";

struct Data {
    client: Client,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let client = Client::try_default().await?;
    let broker = Api::<MqttBroker>::all(client.clone());
    let users = Api::<MqttUser>::all(client.clone());

    let deployments = Api::<Deployment>::all(client.clone());
    let services = Api::<Service>::all(client.clone());
    let configmaps = Api::<ConfigMap>::all(client.clone());

    Controller::new(broker.clone(), watcher::Config::default())
        .owns(deployments, watcher::Config::default())
        .owns(services, watcher::Config::default())
        .owns(configmaps, watcher::Config::default())
        .watches(users, watcher::Config::default(), |user| {
            let broker_name = user.spec.broker_ref.name.clone();
            Some(
                ObjectRef::new(&broker_name)
                    .within(user.namespace().as_deref().expect("user namespace")),
            )
        })
        .shutdown_on_signal()
        .run(
            reconcile::reconcile,
            error_policy,
            Arc::new(Data { client }),
        )
        .for_each(|_| future::ready(()))
        .await;

    Ok(())
}

fn error_policy(_object: Arc<MqttBroker>, err: &M7oError, _ctx: Arc<Data>) -> Action {
    error!("Reconciliation error: {:#?}", err);
    Action::requeue(Duration::from_secs(5))
}
