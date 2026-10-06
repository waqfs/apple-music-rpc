use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

pub const DEFAULT_CONFIG: &str = include_str!("../config.example.toml");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub discord: DiscordConfig,
    pub activity: ActivityConfig,
    pub apple_music: AppleMusicConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordConfig {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityConfig {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleMusicConfig {
    pub artwork_resolution: u32,
    pub timeout_seconds: u64,
    pub query_results_limit: u32,
}

impl Config {
    pub fn default_path() -> Result<PathBuf, String> {
        let home_dir = env::var_os("HOME")
            .ok_or_else(|| "error: HOME env variable is not defined".to_string())?;
        Ok(PathBuf::from(home_dir)
            .join("Library")
            .join("Application Support")
            .join("apple-music-rpc")
            .join("config.toml"))
    }

    pub fn get(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|e| {
                    format!(
                        "error: failed to create config directory {}: {e}",
                        parent.display()
                    )
                })?;
                fs::write(path, DEFAULT_CONFIG).map_err(|e| {
                    format!(
                        "error: failed to write default config to {}: {e}",
                        path.display()
                    )
                })?;
            }
        }

        let raw = fs::read_to_string(path)
            .map_err(|e| format!("error: failed to read config file: {}: {e}", path.display()))?;
        let config: Config = toml::from_str(&raw).map_err(|e| {
            format!(
                "error: failed to parse config file: {}: {e}",
                path.display()
            )
        })?;
        Ok(config)
    }
}
