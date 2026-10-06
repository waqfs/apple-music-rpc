pub const DEFAULT_CONFIG: &str = include_str!("../config.example.toml");

pub struct Config {
    pub discord: DiscordConfig,
    pub activity: ActivityConfig,
    pub apple_music: AppleMusicConfig,
}

pub struct DiscordConfig {}

pub struct ActivityConfig {}

pub struct AppleMusicConfig {}
