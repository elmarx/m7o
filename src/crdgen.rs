use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::CustomResourceExt;
mod v1;

fn print_crd(crd: &CustomResourceDefinition) {
    print!("---\n{}", serde_yaml::to_string(crd).unwrap());
}

fn main() {
    print_crd(&v1::MqttBroker::crd());
    print_crd(&v1::MqttUser::crd());
}
