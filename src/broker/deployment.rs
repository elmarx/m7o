use crate::util::ContentHashExt;
use crate::v1::MqttBroker;
use crate::{MOSQUITTO_VERSION, labels};
use k8s_openapi::api::apps::v1::{Deployment, DeploymentSpec};
use k8s_openapi::api::core::v1::{
    ConfigMap, ConfigMapVolumeSource, Container, ContainerPort, PodSecurityContext, PodSpec,
    PodTemplateSpec, Secret, SecretVolumeSource, Volume, VolumeMount,
};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{LabelSelector, ObjectMeta};
use kube::Resource;
use kube::runtime::reflector::Lookup;
use std::collections::BTreeMap;

fn pod_annotations(cm_hash: &str, secret_hash: &str) -> BTreeMap<String, String> {
    let mut annotations = BTreeMap::new();
    annotations.insert(
        "m7o.athmer.cloud/config-hash".to_string(),
        cm_hash.to_string(),
    );
    annotations.insert(
        "m7o.athmer.cloud/secret-hash".to_string(),
        secret_hash.to_string(),
    );
    annotations
}

impl MqttBroker {
    #[must_use]
    pub fn deployment(&self, cm: &ConfigMap, s: &Secret) -> Deployment {
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
                        annotations: Some(pod_annotations(&cm.hash(), &s.hash())),
                        ..ObjectMeta::default()
                    }),
                    spec: Some(PodSpec {
                        security_context: Some(PodSecurityContext {
                            fs_group: Some(1883),
                            ..Default::default()
                        }),
                        containers: vec![Container {
                            name: "mosquitto".to_string(),
                            image: format!("eclipse-mosquitto:{MOSQUITTO_VERSION}").into(),
                            ports: Some(vec![ContainerPort {
                                container_port: 1883,
                                name: Some("mqtt".to_string()),
                                ..ContainerPort::default()
                            }]),
                            volume_mounts: Some(vec![
                                VolumeMount {
                                    name: "m7o-cfg".to_string(),
                                    mount_path: "/mosquitto/config/mosquitto.conf".to_string(),
                                    sub_path: Some("mosquitto.conf".to_string()),
                                    ..Default::default()
                                },
                                VolumeMount {
                                    name: "m7o-password-file".to_string(),
                                    mount_path: "/mosquitto/config/password_file".to_string(),
                                    sub_path: Some("password_file".to_string()),

                                    ..Default::default()
                                },
                            ]),
                            ..Container::default()
                        }],
                        volumes: Some(vec![
                            Volume {
                                name: "m7o-cfg".to_string(),
                                config_map: Some(ConfigMapVolumeSource {
                                    name: name.clone(),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            },
                            Volume {
                                name: "m7o-password-file".to_string(),
                                secret: Some(SecretVolumeSource {
                                    secret_name: s.name().map(|n| n.to_string()),
                                    default_mode: Some(0o400),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            },
                        ]),
                        ..PodSpec::default()
                    }),
                },
                ..DeploymentSpec::default()
            }),
            ..Deployment::default()
        }
    }
}
