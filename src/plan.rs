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

#[cfg(test)]
mod tests {

    use super::*;
    use crate::user::BROKER_REF_LABEL;
    use crate::v1::{BrokerRef, BrokerSpec, MqttUserSpec, ServiceConfig};
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
    use std::collections::BTreeMap;

    #[test]
    fn test_plan() {
        let sample = MqttBroker {
            metadata: ObjectMeta {
                name: Some("test-broker".to_string()),
                uid: Some("broker-uid".to_string()),
                namespace: Some("test-ns".to_string()),
                ..Default::default()
            },
            spec: BrokerSpec {
                desc: "test".to_string(),
                service: Default::default(),
            },
        };

        let (cm, deploy, svc, secrets) = plan(&sample, &[], &[]);

        assert_eq!(cm.name_any(), "test-broker");
        assert_eq!(deploy.name_any(), "test-broker");
        assert_eq!(svc.name_any(), "test-broker");
        assert!(secrets.is_empty());
    }

    #[test]
    fn test_create_user() {
        let sample_broker = MqttBroker {
            metadata: ObjectMeta {
                name: Some("test-broker".to_string()),
                uid: Some("broker-uid".to_string()),
                namespace: Some("test-ns".to_string()),
                ..Default::default()
            },
            spec: BrokerSpec {
                desc: "test".to_string(),
                service: ServiceConfig::default(),
            },
        };

        let sample_user = MqttUser {
            metadata: ObjectMeta {
                name: Some("test-user".to_string()),
                uid: Some("user-uid".to_string()),
                namespace: Some("test-ns".to_string()),
                ..Default::default()
            },
            spec: MqttUserSpec {
                broker_ref: BrokerRef {
                    name: "test-broker".to_string(),
                },
                username: "myuser".to_string(),
            },
        };

        let (_, _, _, secrets) = plan(&sample_broker, &[sample_user], &[]);

        let (head, tails) = secrets.split_first().unwrap();

        assert!(tails.is_empty());
        assert_eq!(head.name_any(), "test-broker-myuser");
        let data = head.string_data.as_ref().unwrap();
        assert!(data.contains_key("password"));
        assert_eq!(data.get("username").unwrap(), "myuser");
    }

    #[test]
    fn test_existing_secret() {
        let sample_broker = MqttBroker {
            metadata: ObjectMeta {
                name: Some("test-broker".to_string()),
                uid: Some("broker-uid".to_string()),
                namespace: Some("test-ns".to_string()),
                ..Default::default()
            },
            spec: BrokerSpec {
                desc: "test".to_string(),
                service: ServiceConfig::default(),
            },
        };

        let sample_user = MqttUser {
            metadata: ObjectMeta {
                name: Some("test-user".to_string()),
                uid: Some("user-uid".to_string()),
                namespace: Some("test-ns".to_string()),
                ..Default::default()
            },
            spec: MqttUserSpec {
                broker_ref: BrokerRef {
                    name: "test-broker".to_string(),
                },
                username: "myuser".to_string(),
            },
        };

        let mut data = BTreeMap::new();
        data.insert("password".into(), "given password".into());

        let existing_secret = Secret {
            metadata: ObjectMeta {
                name: Some("test-broker-myuser".to_string()),
                namespace: Some("test-ns".to_string()),
                labels: Some(
                    [(BROKER_REF_LABEL.to_string(), "test-broker".to_string())]
                        .into_iter()
                        .collect(),
                ),
                ..Default::default()
            },
            string_data: Some(data),
            ..Default::default()
        };

        let (_, _, _, actual_secrets) = plan(&sample_broker, &[sample_user], &[existing_secret]);

        assert!(actual_secrets.is_empty());
    }
}
