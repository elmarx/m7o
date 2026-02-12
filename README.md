# m7o — mosquitto mqtt operator for Kubernetes

An operator to deploy and manage mosquitto mqtt broker.

## Features

- [x] MosquittoBroker CRD to deploy broker
- [x] MosquittoUser to create users (and store credentials to specified secret)
- [ ] MosquittoAcl to control access
- [ ] MosquittoBridge to mirror topics from/to other brokers

# Local testing with minikube

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

# Debugging

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