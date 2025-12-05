use kube::CustomResourceExt;
mod v1;

fn main() {
    print!("{}", serde_yaml::to_string(&v1::MqttBroker::crd()).unwrap());
}
