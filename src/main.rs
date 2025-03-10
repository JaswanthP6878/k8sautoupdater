use tokio::{sync::mpsc::{self, Sender}, task};

use autoupdater::AutoUpdater;
use axum::{routing::post, Json, Router};
use serde::{Deserialize, Serialize};
mod autoupdater;
use log::info;
use log4rs;


#[derive(Debug, Deserialize, Serialize)]
struct DockerHubWebhook {
    repository: Repository,
    push_data: PushData,
}

#[derive(Debug, Deserialize, Serialize)]
struct Repository {
    repo_name: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct PushData {
    tag: String,
}

/// Handle incoming webhook requests
async fn handle_webhook(Json(payload): Json<DockerHubWebhook>, tx: Sender<DockerHubWebhook>) {
    // let repo_name = payload.repository.repo_name;
    // let tag = payload.push_data.tag;
    // info!("Received webhook: Repository - {}, Tag - {}", repo_name, tag);
    tx.send(payload).await.expect("cannot send value to payload");
}


#[tokio::main]
async fn main() {
    log4rs::init_file("log4rs.yaml", Default::default()).expect("Failed to initialize logger");
    info!("Starting Kubernetes Deployment scanner...");
    let mut autoupdater = AutoUpdater::new();
    let (tx, rx) = mpsc::channel::<DockerHubWebhook>(10);
    // autoupdater.init(rx).await;
    task::spawn(async move {
        autoupdater.init(rx).await;
    });

    // running a router to simultaneosly have a web hook to docker hub so that we can recieve request;
    let app = Router::new().route("/webhook", post(|json| handle_webhook(json, tx)));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("started webHook server at port 3000");
    axum::serve(listener, app).await.unwrap();
}
