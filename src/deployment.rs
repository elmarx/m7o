use crate::v1::MqttBroker;
use crate::{MOSQUITTO_VERSION, labels};
use k8s_openapi::api::apps::v1::{Deployment, DeploymentSpec};
use k8s_openapi::api::core::v1::{Container, ContainerPort, PodSpec, PodTemplateSpec};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{LabelSelector, ObjectMeta};
use kube::Resource;

impl MqttBroker {
    #[must_use]
    pub fn deployment(&self) -> Deployment {
        let oref = self.controller_owner_ref(&()).unwrap();
        let labels = labels::metadata(self.metadata.name.as_ref().unwrap(), MOSQUITTO_VERSION);
        let selector = labels::selector(self.metadata.name.as_ref().unwrap());

        Deployment {
            metadata: ObjectMeta {
                name: self.metadata.name.clone(),
                namespace: self.metadata.namespace.clone(),
                owner_references: Some(vec![oref]),
                labels: Some(labels.clone()),
                ..ObjectMeta::default()
            },
            spec: Some(DeploymentSpec {
                selector: LabelSelector {
                    match_labels: Some(selector),
                    ..LabelSelector::default()
                },
                template: PodTemplateSpec {
                    metadata: Some(ObjectMeta {
                        labels: Some(labels),
                        ..ObjectMeta::default()
                    }),
                    spec: Some(PodSpec {
                        containers: vec![Container {
                            name: "mosquitto".to_string(),
                            image: format!("eclipse-mosquitto:{MOSQUITTO_VERSION}").into(),
                            ports: Some(vec![ContainerPort {
                                container_port: 1883,
                                name: Some("mqtt".to_string()),
                                ..ContainerPort::default()
                            }]),
                            ..Container::default()
                        }],
                        ..PodSpec::default()
                    }),
                },
                ..DeploymentSpec::default()
            }),
            ..Deployment::default()
        }
    }
}
