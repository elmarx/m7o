# m7o — mosquitto mqtt operator for Kubernetes

An operator to deploy and manage mosquitto mqtt broker.

## Features

- [x] MosquittoBroker CRD to deploy broker
- [x] MosquittoUser to create users (and store credentials to specified secret)
- [ ] MosquittoAcl to control access
- [ ] MosquittoBridge to mirror topics from/to other brokers

# Installation

```shell
helm install --create-namespace --namespace m7o m7o oci://ghcr.io/elmarx/charts/m7o
```

# Usage

### Define a MosquittoBroker resource:

```yaml
apiVersion: m7o.athmer.cloud/v1alpha1
kind: MqttBroker
metadata:
  name: homassistant
  namespace: my-namespace
spec:
  desc: "Home Assistant Broker"
  service:
    type: LoadBalancer
```

### define user(s) for the broker:

```yaml
apiVersion: m7o.athmer.cloud/v1alpha1
kind: MqttUser
metadata:
  name: my-user
  namespace: my-namespace
spec:
  brokerRef:
    name: homassistant
  username: my-user
```

### use the generated password

Find the user's credentials in secret `$BROKER_NAME-$USERNAME` (e.g. "homassistant-my-user")

# Development

## Local testing with minikube

```shell
minikube start
# generate and install CRDs
cargo run --bin crdgen | kubectl --context {{ context }} apply -f -
# install sample broker
kubectl apply -f examples/manifests
# run m7o operator
cargo run
# make minikube loadbalancers available
minikube tunnel 
# get the broker's external IP
IP=$(kubectl get svc mosquitto -o jsonpath='{.status.loadBalancer.ingress[0].ip}')
# get the generated password
PASSWORD=$(kubectl get secrets mosquitto-elmar -o jsonpath="{.data.password}" | base64 -d)  
# sub to a topic
mosquitto_sub -h $IP -u elmar -P $PASSWORD -t "m7o"
# publish a message
mosquitto_pub -h $IP -u elmar -P $PASSWORD -t "m7o" -m "Hello, m7o!"
```

## Debugging

Check if CRD/sample is installed

```shell
kubectl get crd mqttbrokers.m7o.athmer.cloud
kubectl get mqttbrokers -n default
```

Check if the Controller/service-account/current user has permissions

```
kubectl auth can-i list mqttbrokers.m7o.athmer.cloud
kubectl auth can-i watch mqttbrokers.m7o.athmer.cloud
```

# License

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.