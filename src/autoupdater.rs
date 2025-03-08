use std::collections::HashSet;
use k8s_openapi::api::core::v1::Pod;
use kube::{api::ListParams, Api, Client, ResourceExt};

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
    pub async fn init_updater(&mut self) {
        // get all pods first
        let client = Client::try_default().await.unwrap();
        let pods: Api<Pod> = Api::default_namespaced(client);
        for p in pods.list(&ListParams::default()).await.unwrap(){ 
            println!("found pod {}", p.name_any());
        }
    }

}