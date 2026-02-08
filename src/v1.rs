use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[kube(
    kind = "MqttBroker",
    group = "m7o.athmer.cloud",
    version = "v1alpha",
    namespaced
)]
pub struct BrokerSpec {
    pub desc: String,
    #[serde(default)]
    pub service: ServiceConfig,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, Default)]
pub struct ServiceConfig {
    #[serde(rename = "type", default)]
    pub type_: ServiceType,
    #[serde(default)]
    pub annotations: BTreeMap<String, String>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, Default, strum::Display)]
pub enum ServiceType {
    #[default]
    ClusterIP,
    LoadBalancer,
}

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[kube(
    kind = "MqttUser",
    group = "m7o.athmer.cloud",
    version = "v1alpha",
    namespaced
)]
#[serde(rename_all = "camelCase")]
pub struct MqttUserSpec {
    pub broker_ref: BrokerRef,
    pub username: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct BrokerRef {
    pub name: String,
}
