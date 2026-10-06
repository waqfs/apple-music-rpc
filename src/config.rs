use std::{env, path::PathBuf};

pub const DEFAULT_CONFIG: &str = include_str!("../config.example.toml");

pub struct Config {
    pub discord: DiscordConfig,
    pub activity: ActivityConfig,
    pub apple_music: AppleMusicConfig,
}

pub struct DiscordConfig {}

pub struct ActivityConfig {}

pub struct AppleMusicConfig {}

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
}
