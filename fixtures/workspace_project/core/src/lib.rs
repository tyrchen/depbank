use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CoreConfig {
    pub name: String,
    pub version: String,
    pub enabled: bool,
}

pub async fn process_config(config: &CoreConfig) -> Result<String, String> {
    if !config.enabled {
        return Err("Config is disabled".to_string());
    }

    log::info!("Processing config: {}", config.name);

    Ok(format!(
        "Processed {} version {}",
        config.name, config.version
    ))
}
