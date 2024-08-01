use anyhow::{anyhow, Context};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Deserialize, Serialize, Default)]
pub struct Config {
    pub current_server: Option<String>,
    pub servers: IndexMap<String, ServerConfig>,
}

impl Config {
    pub fn load_or_create() -> anyhow::Result<Self> {
        let path = config_path();
        if path.exists() {
            let raw_config = fs::read_to_string(&path).context("Could not read config")?;
            let config = serde_json::from_str(&raw_config).context("Could not parse config")?;
            Ok(config)
        } else {
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = config_path();
        let config_folder = path.parent().unwrap();
        fs::create_dir_all(config_folder).context("Could not create config dir")?;

        let data = serde_json::to_string_pretty(self).unwrap();
        fs::write(&path, data).context("Could not write config")?;
        Ok(())
    }

    pub fn get_current_server(&self) -> anyhow::Result<&ServerConfig> {
        self.current_server
            .as_ref()
            .and_then(|current| self.servers.get(current))
            .ok_or_else(|| anyhow!("No server currently selected"))
    }
}

#[derive(Deserialize, Serialize)]
pub struct ServerConfig {
    pub url: String,
}

fn config_path() -> PathBuf {
    dirs::config_dir()
        .expect("Could not read config dir path")
        .join("kodjin-cli")
        .join("config.json")
}
