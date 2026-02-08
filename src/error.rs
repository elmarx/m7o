#[derive(thiserror::Error, Debug)]
pub enum M7oError {
    #[error("Failed to list secrets: {0}")]
    ListSecrets(#[source] kube::Error),
    #[error("Failed to list users: {0}")]
    ListUsers(#[source] kube::Error),
    #[error("Failed to create secret: {0}")]
    CreateSecret(#[source] kube::Error),
    #[error("Failed to patch Secret: {0}")]
    PatchSecret(#[source] kube::Error),
    #[error("Failed to patch deployment: {0}")]
    PatchDeployment(#[source] kube::Error),
    #[error("Failed to patch service: {0}")]
    PatchService(#[source] kube::Error),
    #[error("Failed to patch configmap: {0}")]
    PatchConfigMap(#[source] kube::Error),
}
