# Reel

Update k8s deployments in a cluster, based changes to repo in dockerhub. The application contains a webserver which can be added as a webhook in k8s repo. any update to repo(new version of image), triggres the auto updater to apply the changes to the deployment and update the deployment.

a sample deployment that can be picked up by reel for auto updates is

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: sample-app
  annotations:
    reel: "true"  # Annotation for enabling auto updation
spec:
  replicas: 1
  selector:
    matchLabels:
      app: sample-app
  template:
    metadata:
      labels:
        app: sample-app
      annotations:
        reel: "true"  
    spec:
      containers:
        - name: sample-app
          image: samplerepo/servicea:v1
          ports:
            - containerPort: 8080

```
---
## Building the image and installation

clone the repository and build the image. requires rust 2021
```bash
mkdir -p reel
git clone https://github.com/JaswanthP6878/reel.git .

cargo build --release

docker build -f dockerfile.server  <repo_name>/reel:v1 .
```
---
# limitations
- selective updates not present, updates all the pods that use the repo
- to add new deployments for reel to autoupdate, need to restart the reel pod

--- 
## Status
- [x] supports deployments
- [x] supports webhook server which can be configured on dockerhub
- [ ] support for pods
- [ ] support for selective updates using tag name
- [ ] check for new deployments with reel enabled without restarting pod