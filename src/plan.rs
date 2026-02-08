use crate::credentials::Credentials;
use crate::v1::{MqttBroker, MqttUser};
use itertools::Itertools;
use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::core::v1::{ConfigMap, Secret, Service};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::runtime::reflector::Lookup;
use kube::{Resource, ResourceExt};
use std::collections::HashMap;

pub fn plan(
    broker: &MqttBroker,
    users: &[MqttUser],
    existing_secrets: &[Secret],
) -> (ConfigMap, Deployment, Service, Vec<Secret>, Secret) {
    let mut credentials = existing_secrets
        .iter()
        .filter_map(|s| {
            let credentials = Credentials::try_from(s).ok();
            let name = s.name().map(|n| n.to_string());

            name.and_then(|n| credentials.map(|p| (n, p)))
        })
        .collect::<HashMap<_, _>>();

    let mut secrets_to_create = Vec::with_capacity(users.len() - existing_secrets.len());

    let users = users
        .iter()
        .filter(|u| u.spec.broker_ref.name == broker.name_any());

    for user in users {
        let secret_name = user.secret_name();
        if !credentials.contains_key(secret_name.as_str()) {
            let secret = user.secret();
            credentials.insert(secret_name, (&secret).try_into().unwrap());
            secrets_to_create.push(secret);
        }
    }

    let password_file = credentials
        .values()
        .sorted_by_key(|c| &c.username)
        .map(Credentials::password_file_line)
        .collect::<Vec<String>>();

    let password_file = Secret {
        metadata: ObjectMeta {
            name: broker.name().map(|n| n.to_string()),
            namespace: broker.namespace().to_string().into(),
            owner_references: Some(vec![broker.controller_owner_ref(&()).unwrap()]),
            ..Default::default()
        },
        string_data: Some(
            [("password_file".to_string(), password_file.concat())]
                .into_iter()
                .collect(),
        ),
        ..Default::default()
    };

    let configmap = broker.configmap();
    let deployment = broker.deployment(&configmap, &password_file);
    let service = broker.service();

    (
        configmap,
        deployment,
        service,
        secrets_to_create,
        password_file,
    )
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

        let (cm, deploy, svc, secrets, _) = plan(&sample, &[], &[]);

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

        let (_, _, _, secrets, _) = plan(&sample_broker, &[sample_user], &[]);

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
        data.insert("username".into(), "myuser".into());
        data.insert("hash".into(), "hashed".into());

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

        let (_, _, _, actual_secrets, _) = plan(&sample_broker, &[sample_user], &[existing_secret]);

        assert!(actual_secrets.is_empty());
    }
}
