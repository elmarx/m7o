use crate::labels;
use crate::v1::MqttBroker;
use k8s_openapi::api::core::v1::{Service, ServiceSpec};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;
use kube::Resource;

impl MqttBroker {
    #[must_use]
    pub fn service(&self) -> Service {
        let oref = self.controller_owner_ref(&()).unwrap();
        let labels = labels::selector(self.metadata.name.as_ref().unwrap());
        let selector = labels::selector(self.metadata.name.as_ref().unwrap());

        Service {
            metadata: ObjectMeta {
                name: self.metadata.name.clone(),
                namespace: self.metadata.namespace.clone(),
                owner_references: Some(vec![oref]),
                labels: Some(labels),
                ..ObjectMeta::default()
            },
            spec: ServiceSpec {
                selector: Some(selector),
                ports: Some(vec![k8s_openapi::api::core::v1::ServicePort {
                    port: 1883,
                    target_port: Some(IntOrString::String("mqtt".to_string())),
                    ..k8s_openapi::api::core::v1::ServicePort::default()
                }]),
                ..ServiceSpec::default()
            }
            .into(),
            status: None,
        }
    }
}
