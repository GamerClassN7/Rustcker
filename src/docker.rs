use bollard::{Docker, container::{StartContainerOptions, StopContainerOptions, InspectContainerOptions}};
use std::{time::Instant, sync::Arc};
use crate::state::AppState;
use tokio::time::{sleep, Duration};
use anyhow::Result;

pub async fn ensure_container_running(name: &str, state: &Arc<AppState>) -> Result<()> {
    {
        let cache = state.container_cache.read().await;
        if cache.get(name).copied() == Some(true) {
            return Ok(());
        }
    }

    let docker = Docker::connect_with_unix_defaults()?;
    let info = docker.inspect_container(name, None::<InspectContainerOptions>).await?;
    if let Some(s) = info.state {
        if !s.running.unwrap_or(false) {
            docker.start_container(name, None::<StartContainerOptions<String>>).await?;
            println!("Started container: {}", name);
        }
    }

    let mut cache = state.container_cache.write().await;
    cache.insert(name.to_string(), true);
    Ok(())
}

pub fn start_idle_watcher(state: Arc<AppState>) {
    tokio::spawn(async move {
        let docker = Docker::connect_with_unix_defaults().expect("docker connect failed");
        loop {
            sleep(Duration::from_secs(30)).await;
            let now = Instant::now();

            for route in &state.config.routes {
                if let (Some(name), Some(timeout)) = (&route.container, route.idle_timeout) {
                    let mut last = state.last_activity.write().await;
                    if let Some(last_used) = last.get(name) {
                        if now.duration_since(*last_used).as_secs() >= timeout {
                            println!("Stopping idle container: {}", name);
                            let _ = docker.stop_container(name, Some(StopContainerOptions { t: 5 })).await;
                            last.remove(name);
                            let mut cache = state.container_cache.write().await;
                            cache.insert(name.to_string(), false);
                        }
                    }
                }
            }
        }
    });
}