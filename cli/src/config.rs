use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub server_url: String,
    pub token: String,
}

impl Config {
    pub fn path() -> Result<PathBuf> {
        let dir = dirs::config_dir()
            .context("could not find config directory")?
            .join("centralenv");
        std::fs::create_dir_all(&dir)?;
        Ok(dir.join("config.toml"))
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path()?;
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        println!("Config saved to {}", path.display());
        Ok(())
    }

    pub fn load() -> Result<Self> {
        let path = Self::path()?;
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Config not found at {}. Run `centralenv login` first.", path.display()))?;
        toml::from_str(&content).context("failed to parse config")
    }
}
