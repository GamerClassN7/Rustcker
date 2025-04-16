mod config;
mod proxy;
mod docker;
mod state;

use axum::Router;
use config::load_config;
use docker::start_idle_watcher;
use proxy::proxy_handler;
use state::AppState;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use axum::extract::Extension;
use tower::service_fn;

#[tokio::main]
async fn main() {
    let config = load_config("config.yaml");
    let state = Arc::new(AppState::new(config));

    start_idle_watcher(state.clone());

    let state_clone = state.clone();
    let app = Router::new()
        .fallback_service(service_fn(move |req| {
            proxy_handler(req, Extension(state_clone.clone()))
        }))
        .layer(ServiceBuilder::new().layer(Extension(state.clone())));

    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("Listening on http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}