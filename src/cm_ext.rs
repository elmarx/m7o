use k8s_openapi::api::core::v1::ConfigMap;
use std::hash::{DefaultHasher, Hash, Hasher};

pub trait ConfigMapExt {
    fn hash(&self) -> String;
}

impl ConfigMapExt for ConfigMap {
    fn hash(&self) -> String {
        let mut hasher = DefaultHasher::new();

        if let Some(data) = &self.data {
            data.hash(&mut hasher);
        }

        let hash = hasher.finish();

        format!("{hash:x}")
    }
}
