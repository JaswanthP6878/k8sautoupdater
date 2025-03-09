use std::collections::HashSet;
use k8s_openapi::{api::{apps::v1::Deployment, core::v1::Pod}, List};
use kube::{api::ListParams, Api, Client, ResourceExt};

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
        let lp = ListParams::default().labels("reel=true");
        for deploy in deployments.list(&lp).await.unwrap().items{
            if let Some(name) = deploy.metadata.name {
                info!("Found Deployment: {}", name);
            }
        }
    }

}