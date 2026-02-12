crd context="minikube":
    cargo run --bin crdgen | kubectl --context {{ context }} apply -f -

# run [helm-docs](https://github.com/norwoodj/helm-docs)
docs:
    helm-docs
