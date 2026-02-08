use crate::cm_ext::ConfigMapExt;
use crate::v1::MqttBroker;
use crate::{MOSQUITTO_VERSION, labels};
use k8s_openapi::api::apps::v1::{Deployment, DeploymentSpec};
use k8s_openapi::api::core::v1::{ConfigMap, Container, ContainerPort, PodSpec, PodTemplateSpec};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{LabelSelector, ObjectMeta};
use kube::Resource;
use std::collections::BTreeMap;

fn pod_annotations(hash: &str) -> BTreeMap<String, String> {
    let mut annotations = BTreeMap::new();
    annotations.insert("m7o.athmer.cloud/config-hash".to_string(), hash.to_string());
    annotations
}

impl MqttBroker {
    #[must_use]
    pub fn deployment(&self, cm: &ConfigMap) -> Deployment {
        let oref = self.controller_owner_ref(&()).unwrap();
        let name = self.metadata.name.as_ref().unwrap();
        let labels = labels::metadata(name, MOSQUITTO_VERSION);
        let selector = labels::selector(name);

        Deployment {
            metadata: ObjectMeta {
                name: name.to_string().into(),
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
                        annotations: Some(pod_annotations(&cm.hash())),
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
                            volume_mounts: Some(vec![k8s_openapi::api::core::v1::VolumeMount {
                                name: "m7o-cfg".to_string(),
                                mount_path: "/mosquitto/config".to_string(),
                                ..Default::default()
                            }]),
                            ..Container::default()
                        }],
                        volumes: Some(vec![k8s_openapi::api::core::v1::Volume {
                            name: "m7o-cfg".to_string(),
                            config_map: Some(k8s_openapi::api::core::v1::ConfigMapVolumeSource {
                                name: name.clone(),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }]),
                        ..PodSpec::default()
                    }),
                },
                ..DeploymentSpec::default()
            }),
            ..Deployment::default()
        }
    }
}
