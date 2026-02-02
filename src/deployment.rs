use crate::v1::MqttBroker;
use crate::{MOSQUITTO_VERSION, labels};
use k8s_openapi::api::apps::v1::{Deployment, DeploymentSpec};
use k8s_openapi::api::core::v1::{
    ConfigMapVolumeSource, Container, ContainerPort, PodSpec, PodTemplateSpec, Volume, VolumeMount,
};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{LabelSelector, ObjectMeta};
use kube::Resource;

impl MqttBroker {
    #[must_use]
    pub fn deployment(&self) -> Deployment {
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
                        ..ObjectMeta::default()
                    }),
                    spec: Some(PodSpec {
                        share_process_namespace: Some(true),
                        containers: vec![
                            Container {
                                name: "mosquitto".to_string(),
                                image: format!("eclipse-mosquitto:{MOSQUITTO_VERSION}").into(),
                                ports: Some(vec![ContainerPort {
                                    container_port: 1883,
                                    name: Some("mqtt".to_string()),
                                    ..ContainerPort::default()
                                }]),
                                volume_mounts: Some(vec![VolumeMount {
                                    name: "m7o-cfg".to_string(),
                                    mount_path: "/mosquitto/config".to_string(),
                                    ..Default::default()
                                }]),
                                ..Container::default()
                            },
                            // a sidecar that sends SIGHUP to mosquitto if the configmap/mosquitto.conf changes
                            Container {
                                name: "watcher".to_string(),
                                image: Some("alpine".to_string()),
                                command: Some(vec![
                                    "sh".to_string(),
                                    "-c".to_string(),
                                    "echo -e '#!/bin/sh\\nkill -HUP $(pidof mosquitto)' > /usr/local/bin/reload.sh && chmod +x /usr/local/bin/reload.sh && while :; do inotifyd /usr/local/bin/reload.sh /mosquitto/config/mosquitto.conf:c; sleep 1; done".to_string(),
                                ]),
                                volume_mounts: Some(vec![
                                    VolumeMount {
                                        name: "m7o-cfg".to_string(),
                                        mount_path: "/mosquitto/config".to_string(),
                                        ..Default::default()
                                    },
                                ]),
                                ..Container::default()
                            },
                        ],
                        volumes: Some(vec![
                            Volume {
                                name: "m7o-cfg".to_string(),
                                config_map: Some(ConfigMapVolumeSource {
                                    name: name.clone(),
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
