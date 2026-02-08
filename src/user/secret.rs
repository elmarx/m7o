use crate::user::BROKER_REF_LABEL;
use crate::v1::MqttUser;
use k8s_openapi::api::core::v1::Secret;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::Resource;
use std::collections::BTreeMap;

impl MqttUser {
    #[must_use]
    pub fn secret(&self) -> Secret {
        let name = format!("{}-{}", self.spec.broker_ref.name, self.spec.username);

        let password = crate::util::generate_password();
        let mut data = BTreeMap::new();
        data.insert("password".to_string(), password);
        data.insert("username".to_string(), self.spec.username.clone());

        let mut labels = BTreeMap::new();
        labels.insert(
            BROKER_REF_LABEL.to_string(),
            self.spec.broker_ref.name.clone(),
        );

        Secret {
            metadata: ObjectMeta {
                name: Some(name),
                namespace: self.metadata.namespace.clone(),
                owner_references: Some(vec![self.controller_owner_ref(&()).unwrap()]),
                labels: Some(labels),
                ..Default::default()
            },
            string_data: Some(data),
            ..Default::default()
        }
    }
}
