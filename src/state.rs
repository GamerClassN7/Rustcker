use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Instant;
use crate::config::Config;

pub type SharedState = Arc<AppState>;

pub struct AppState {
    pub config: Config,
    pub container_cache: RwLock<HashMap<String, bool>>,
    pub last_activity: RwLock<HashMap<String, Instant>>,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            container_cache: RwLock::new(HashMap::new()),
            last_activity: RwLock::new(HashMap::new()),
        }
    }
}
