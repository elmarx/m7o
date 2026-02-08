use crate::MOSQUITTO_VERSION;
use crate::labels;
use crate::v1::MqttBroker;
use k8s_openapi::api::core::v1::ConfigMap;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::Resource;
use std::collections::BTreeMap;

// default config from docker
const MOSQUITTO_CONF: &str = r#"
listener 1883
password_file /mosquitto/config/password_file

listener 9883
protocol http_api
http_dir /usr/share/mosquitto/dashboard
"#;

impl MqttBroker {
    #[must_use]
    pub fn configmap(&self) -> ConfigMap {
        let oref = self.controller_owner_ref(&()).unwrap();
        let labels = labels::metadata(self.metadata.name.as_ref().unwrap(), MOSQUITTO_VERSION);

        let mut data = BTreeMap::new();
        data.insert(
            "mosquitto.conf".to_string(),
            MOSQUITTO_CONF.trim().to_string(),
        );

        ConfigMap {
            metadata: ObjectMeta {
                name: self.metadata.name.clone(),
                namespace: self.metadata.namespace.clone(),
                owner_references: Some(vec![oref]),
                labels: Some(labels),
                ..ObjectMeta::default()
            },
            data: Some(data),
            ..ConfigMap::default()
        }
    }
}
