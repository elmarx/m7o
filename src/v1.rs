use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[kube(
    kind = "MqttBroker",
    group = "m7o.athmer.cloud",
    version = "v1alpha",
    namespaced
)]
pub struct BrokerSpec {
    desc: String,
}
