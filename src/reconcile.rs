use crate::error::M7oError;
use crate::v1::{MqttBroker, MqttUser};
use crate::{Data, MANAGER, plan};
use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::core::v1::{ConfigMap, Secret, Service};
use kube::api::{ListParams, PatchParams, PostParams};
use kube::runtime::controller::Action;
use kube::{Api, ResourceExt};
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

pub async fn reconcile(obj: Arc<MqttBroker>, ctx: Arc<Data>) -> crate::Result<Action> {
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
        .items;

    // List all secrets managed by this operator for this broker
    let lp = ListParams::default().labels(&format!(
        "{}={}",
        crate::user::BROKER_REF_LABEL,
        obj.name_any()
    ));

    let existing_secrets = secrets_api
        .list(&lp)
        .await
        .map_err(M7oError::ListSecrets)?
        .items;

    let (configmap, deployment, service, secrets_to_create) =
        plan::plan(&obj, &users, &existing_secrets);

    for secret in secrets_to_create {
        secrets_api
            .create(&PostParams::default(), &secret)
            .await
            .map_err(M7oError::CreateSecret)?;
    }

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
