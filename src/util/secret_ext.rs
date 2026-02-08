use crate::util::ContentHashExt;
use k8s_openapi::api::core::v1::Secret;
use std::hash::{DefaultHasher, Hash, Hasher};

impl ContentHashExt for Secret {
    fn hash(&self) -> String {
        let mut hasher = DefaultHasher::new();

        if let Some(data) = &self.data {
            for (k, v) in data {
                k.hash(&mut hasher);
                v.0.hash(&mut hasher);
            }
        }

        if let Some(string_data) = &self.string_data {
            string_data.hash(&mut hasher);
        }

        let hash = hasher.finish();

        format!("{hash:x}")
    }
}
