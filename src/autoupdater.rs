use std::collections::HashSet;
use k8s_openapi::api::apps::v1::Deployment;
use kube::{api::ListParams, Api, Client};

use log::info;

pub struct Autoupdater {
    resourses: HashSet<String>,
}

impl Autoupdater {
    pub fn new() -> Self {
        Self {
            resourses: HashSet::new(),
        }
    }

    // here we check for all the resources that we want to 
    // autoupdate the resource for.
    // 
    pub async fn init_updater(&mut self) {
        // get all pods first
        let client = Client::try_default().await.unwrap();
        let deployments: Api<Deployment> = Api::all(client);

        let lp = ListParams::default();
        for deploy in deployments.list(&lp).await.unwrap().items {
            if let Some(annotations) = &deploy.metadata.annotations {
                if annotations.get("reel").map(|v: &String| v == "true").unwrap_or(false) {
                    if let Some(name) = deploy.metadata.name {
                        info!("Found Deployment with reel=true: {}", name);
                    }
                }
            }
        }
    }

}