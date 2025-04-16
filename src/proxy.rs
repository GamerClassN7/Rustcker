use axum::{
    extract::Extension,
    http::{Request, StatusCode, Uri},
    response::{IntoResponse, Response},
    body::Body,
};
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use hyper_util::client::legacy::connect::HttpConnector;
use std::sync::Arc;
use crate::state::AppState;
use crate::docker::ensure_container_running;

pub async fn proxy_handler(
    req: Request<Body>,
    Extension(state): Extension<Arc<AppState>>,
) -> Response {
    let path = req.uri().path();
    let route = state.config.routes.iter()
        .find(|r| path.starts_with(&r.path_prefix));

    if let Some(route) = route {
        if let Some(container) = &route.container {
            if let Err(e) = ensure_container_running(container, &state).await {
                eprintln!("Failed to start container '{}': {}", container, e);
                return StatusCode::BAD_GATEWAY.into_response();
            }
        }

        let new_uri = format!(
            "{}{}",
            route.target.trim_end_matches('/'),
            path.trim_start_matches(&route.path_prefix)
        );
        let uri: Uri = match new_uri.parse() {
            Ok(u) => u,
            Err(_) => return StatusCode::BAD_GATEWAY.into_response(),
        };

        let (mut parts, body) = req.into_parts();
        parts.uri = uri;
        let proxied_req = Request::from_parts(parts, body);

        let connector = HttpConnector::new();
        let client: Client<_, Body> = Client::builder(TokioExecutor::new()).build(connector);

        match client.request(proxied_req).await {
            Ok(resp) => {
                let (parts, body) = resp.into_parts();
                Response::from_parts(parts, body)
            },
            Err(_) => StatusCode::BAD_GATEWAY.into_response(),
        }
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}
