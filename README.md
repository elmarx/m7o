# m7o — mosquitto mqtt operator for Kubernetes

An operator to deploy and manage mosquitto mqtt broker.

## Features

- [ ] MosquittoBroker CRD to deploy broker
- [ ] MosquittoUser to create users (and store credentials to specified secret)
- [ ] MosquittoAcl to control access
- [ ] MosquittoBridge to mirror topics from/to other brokers

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