use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use k8s_openapi::serde_json::{self, json};
use kube::{api::{ListParams, Patch, PatchParams}, Api, Client};
use k8s_openapi::api::apps::v1::Deployment;
use log::{info, error};
use tokio::sync::{Mutex, mpsc};
use crate::DockerHubWebhook;

#[derive(Eq, Hash, PartialEq)]
struct DeploymentData {
    deployment_name: String,  // Added deployment_name field
    container_name: String,
    tag: String,
    namespace: String,
}

#[derive(Clone)]
pub struct AutoUpdater {
    deployment_map: Arc<Mutex<HashMap<String, HashSet<DeploymentData>>>>,
}

impl AutoUpdater {
    pub fn new() -> Self {
        Self {
            deployment_map: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Initializes and caches deployments with `reel=true` annotation
    async fn init_updater(&mut self) {
        let client = Client::try_default().await.unwrap();
        let deployments: Api<Deployment> = Api::all(client);
        let lp = ListParams::default();
        let mut map = self.deployment_map.lock().await;

        for deploy in deployments.list(&lp).await.unwrap().items {
            if let Some(annotations) = &deploy.metadata.annotations {
                if annotations.get("reel").map(|v| v == "true").unwrap_or(false) {
                    if let (Some(deployment_name), Some(namespace)) = (deploy.metadata.name.clone(), deploy.metadata.namespace.clone()) {
                        let container_info = deploy
                            .spec
                            .as_ref()
                            .and_then(|s| s.template.spec.as_ref())
                            .and_then(|t| t.containers.first());

                        if let Some(container) = container_info {
                            let container_name = container.name.clone();
                            let current_image = container.image.clone().unwrap_or_default();
                            let image_repo: String = current_image.split(':').next().unwrap_or("").to_string();

                            // Create DeploymentData with deployment_name
                            let deployment_data = DeploymentData {
                                deployment_name: deployment_name.clone(),
                                container_name: container_name.clone(),
                                tag: image_repo.clone(),
                                namespace: namespace.clone(),
                            };

                            // Insert into deploymentMap
                            map.entry(image_repo.clone())
                                .or_insert_with(HashSet::new)
                                .insert(deployment_data);

                            info!(
                                "Cached Deployment: {} | Namespace: {} | Container: {} | Image Repo: {}",
                                deployment_name, namespace, container_name, image_repo
                            );
                        }
                    }
                }
            }
        }
    }

    /// Updates the cached deployments with a new image when an event is triggered
    pub async fn update_deployments(&self, repo_name: &str, full_image: &str) {
        let client = Client::try_default().await.unwrap();
        let mut map = self.deployment_map.lock().await;
    
        // Get deployments associated with this repo_name
        if let Some(deployments) = map.get_mut(repo_name) {
            for deployment_data in deployments.iter() {
                let namespace = &deployment_data.namespace;
                let deployment_name = &deployment_data.deployment_name;
                let container_name = &deployment_data.container_name;
    
                let deployments_api: Api<Deployment> = Api::namespaced(client.clone(), namespace);
    
                let patch = json!({
                    "spec": {
                        "template": {
                            "spec": {
                                "containers": [{
                                    "name": container_name,
                                    "image": full_image
                                }]
                            }
                        }
                    }
                });
    
                info!(
                    "Updating Deployment: {} | Namespace: {} | Container: {} | New Image: {}",
                    deployment_name, namespace, container_name, full_image
                );
    
                match deployments_api
                    .patch(deployment_name, &PatchParams::apply("my-updater"), &Patch::Strategic(&patch))
                    .await
                {
                    Ok(_) => info!("Updated deployment {} in namespace {} with new image: {}", deployment_name, namespace, full_image),
                    Err(e) => error!("Failed to update deployment {} in namespace {}: {:?}", deployment_name, namespace, e),
                }
            }
        } else {
            info!("No deployments found for repo: {}", repo_name);
        }
    }
    
    /// Main loop that listens for webhook events
    pub async fn init(&mut self, mut rx: mpsc::Receiver<DockerHubWebhook>) {
        self.init_updater().await;
        while let Some(webhook_data) = rx.recv().await {
            let repo_name = webhook_data.repository.repo_name;
            let tag = webhook_data.push_data.tag;
            info!(
                "Processing webhook: Repository - {}, Tag - {}",
                repo_name, tag
            );
            let full_image = format!("{}:{}", repo_name, tag);
            self.update_deployments(&repo_name, &full_image).await;
        }
    }
}
