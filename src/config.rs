use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Route {
    pub path_prefix: String,
    pub target: String,
    pub container: Option<String>,
    pub idle_timeout: Option<u64>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub routes: Vec<Route>,
}

pub fn load_config(path: &str) -> Config {
    let content = std::fs::read_to_string(path).expect("Failed to read config file");
    serde_yaml::from_str(&content).expect("Failed to parse YAML")
}
