use autoupdater::Autoupdater;
use axum::{routing::get, Router};
mod autoupdater;

#[tokio::main]
async fn main() {
    // maybe we can init state where we first check the cluster for all the pods
    // and deployments that have the annotation.
    let mut  autoupdater = Autoupdater::new();

    // checks for whether a client is being able to be recognized;
    autoupdater.init_updater().await;

    // running a router to simultaneosly have a web hook to docker hub so that we can recieve request;
    let app = Router::new().route("/", get(|| async {"Hello World!"}));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
