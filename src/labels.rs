use crate::MANAGER;
use std::collections::BTreeMap;

pub fn metadata(instance_name: &str, version: &str) -> BTreeMap<String, String> {
    let mut ls = BTreeMap::new();

    ls.insert(
        "app.kubernetes.io/name".to_string(),
        "mosquitto".to_string(),
    );
    ls.insert(
        "app.kubernetes.io/instance".to_string(),
        format!("mosquitto-{instance_name}"),
    );
    ls.insert("app.kubernetes.io/version".to_string(), version.to_string());
    ls.insert(
        "app.kubernetes.io/managed-by".to_string(),
        MANAGER.to_string(),
    );

    ls
}

pub fn selector(instance_name: &str) -> BTreeMap<String, String> {
    let mut ls = BTreeMap::new();

    ls.insert(
        "app.kubernetes.io/name".to_string(),
        "mosquitto".to_string(),
    );

    ls.insert(
        "app.kubernetes.io/instance".to_string(),
        format!("mosquitto-{instance_name}"),
    );

    ls
}
