use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use k8s_openapi::serde_json::{self, json};
use kube::{api::{ListParams, Patch, PatchParams}, Api, Client};
use k8s_openapi::api::apps::v1::Deployment;
// use serde_json::json;
use log::{info, error};

#[derive(Clone)]
pub struct AutoUpdater {
    deployments: Arc<Mutex<HashMap<String, String>>>, // Stores deployment names -> images
}

impl AutoUpdater {
    pub fn new() -> Self {
        Self {
            deployments: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Initializes and caches deployments with `reel=true` annotation
    pub async fn init_updater(&mut self) {
        let client = Client::try_default().await.unwrap();
        let deployments: Api<Deployment> = Api::all(client);

        let lp = ListParams::default();
        let mut map = self.deployments.lock().unwrap();

        for deploy in deployments.list(&lp).await.unwrap().items {
            if let Some(annotations) = &deploy.metadata.annotations {
                if annotations.get("reel").map(|v| v == "true").unwrap_or(false) {
                    if let Some(name) = deploy.metadata.name.clone() {
                        let current_image = deploy
                            .spec
                            .as_ref()
                            .and_then(|s: &k8s_openapi::api::apps::v1::DeploymentSpec| s.template.spec.as_ref())
                            .and_then(|t| t.containers.first())
                            .map(|c| c.image.clone().unwrap_or_default())
                            .unwrap_or_default();

                        map.insert(name.clone(), current_image.clone());
                        info!("Cached Deployment: {} with image: {}", name, current_image);
                    }
                }
            }
        }
    }

    /// Updates the cached deployments with a new image when an event is triggered
    pub async fn update_deployments(&self, new_image: &str) {
        let client = Client::try_default().await.unwrap();
        let deployments: Api<Deployment> = Api::all(client);
        let map = self.deployments.lock().unwrap();

        for (name, _) in map.iter() {
            let patch = json!({
                "spec": {
                    "template": {
                        "spec": {
                            "containers": [{
                                "name": "sample-app",  // Update container name dynamically if needed
                                "image": new_image
                            }]
                        }
                    }
                }
            });

            match deployments.patch(&name, &PatchParams::apply("my-updater"), &Patch::Merge(&patch)).await {
                Ok(_) => info!("Updated deployment {} with new image: {}", name, new_image),
                Err(e) => eprintln!("Failed to update deployment {}: {:?}", name, e),
            }
        }
    }
}
