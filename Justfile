crd context="minikube":
    cargo run --bin crdgen | kubectl --context {{ context }} apply -f -
