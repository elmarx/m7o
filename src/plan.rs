use std::collections::HashSet;

use crate::v1::{MqttBroker, MqttUser};
use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::core::v1::{ConfigMap, Secret, Service};
use kube::ResourceExt;
use kube::runtime::reflector::Lookup;

pub fn plan(
    broker: &MqttBroker,
    users: &[MqttUser],
    existing_secrets: &[Secret],
) -> (ConfigMap, Deployment, Service, Vec<Secret>) {
    let existing_secrets = existing_secrets
        .iter()
        .filter_map(Lookup::name)
        .collect::<HashSet<_>>();

    let configmap = broker.configmap();
    let deployment = broker.deployment(&configmap);
    let service = broker.service();

    let secrets = users
        .iter()
        .filter(|u| u.spec.broker_ref.name == broker.name_any())
        .map(MqttUser::secret)
        .filter(|s| !existing_secrets.contains(s.name().unwrap().as_ref()))
        .collect();

    (configmap, deployment, service, secrets)
}
