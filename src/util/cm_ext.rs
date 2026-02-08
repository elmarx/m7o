use crate::util::ContentHashExt;
use k8s_openapi::api::core::v1::ConfigMap;
use std::hash::{DefaultHasher, Hash, Hasher};

impl ContentHashExt for ConfigMap {
    fn hash(&self) -> String {
        let mut hasher = DefaultHasher::new();

        if let Some(data) = &self.data {
            data.hash(&mut hasher);
        }

        let hash = hasher.finish();

        format!("{hash:x}")
    }
}
