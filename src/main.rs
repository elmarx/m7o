use std::sync::Arc;
use std::time::Duration;

use crate::v1::{MqttBroker, MqttUser};
use error::M7oError;
use futures::{StreamExt, future, stream};
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
mod user;
mod util;
pub mod v1;

pub type Result<T, E = M7oError> = std::result::Result<T, E>;

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
        .run(reconcile, error_policy, Arc::new(Data { client }))
        .for_each(|_| future::ready(()))
        .await;

    Ok(())
}

fn error_policy(_object: Arc<MqttBroker>, err: &M7oError, _ctx: Arc<Data>) -> Action {
    error!("Reconciliation error: {:#?}", err);
    Action::requeue(Duration::from_secs(5))
}

pub const MANAGER: &str = "m7o.athmer.cloud";

async fn reconcile(obj: Arc<MqttBroker>, ctx: Arc<Data>) -> Result<Action> {
    info!("reconcile request: {}", obj.name_any());

    let deployment_api = Api::<Deployment>::namespaced(ctx.client.clone(), obj.namespace());
    let service_api = Api::<Service>::namespaced(ctx.client.clone(), obj.namespace());
    let configmap_api = Api::<ConfigMap>::namespaced(ctx.client.clone(), obj.namespace());
    let users_api = Api::<MqttUser>::namespaced(ctx.client.clone(), obj.namespace());
    let secrets_api = Api::<Secret>::namespaced(ctx.client.clone(), obj.namespace());

    let users = users_api
        .list(&ListParams::default())
        .await
        .map_err(M7oError::ListUsers)?
        .items
        .into_iter()
        .filter(|u| u.spec.broker_ref.name == obj.name_any())
        .collect::<Vec<_>>();

    let secrets_to_create = stream::iter(users)
        .filter_map(|user| {
            let api = secrets_api.clone();
            async move {
                let secret = user.secret();
                let name = secret.metadata.name.as_deref()?;
                match api.get_opt(name).await {
                    Ok(None) => Some(secret),
                    _ => None,
                }
            }
        })
        .collect::<Vec<_>>()
        .await;

    for secret in secrets_to_create {
        secrets_api
            .create(&PostParams::default(), &secret)
            .await
            .map_err(M7oError::CreateSecret)?;
    }

    let configmap = obj.configmap();
    let deployment = obj.deployment(&configmap);
    let service = obj.service();

    deployment_api
        .patch(
            deployment.metadata.name.as_ref().unwrap(),
            &PatchParams::apply(MANAGER),
            &kube::api::Patch::Apply(&deployment),
        )
        .await
        .map_err(M7oError::PatchDeployment)?;

    service_api
        .patch(
            service.metadata.name.as_ref().unwrap(),
            &PatchParams::apply(MANAGER),
            &kube::api::Patch::Apply(&service),
        )
        .await
        .map_err(M7oError::PatchService)?;

    configmap_api
        .patch(
            configmap.metadata.name.as_ref().unwrap(),
            &PatchParams::apply(MANAGER),
            &kube::api::Patch::Apply(&configmap),
        )
        .await
        .map_err(M7oError::PatchConfigMap)?;

    Ok(Action::requeue(Duration::from_secs(3600)))
}
