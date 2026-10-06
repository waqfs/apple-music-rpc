use crate::{config::Config, music::TrackState};

pub struct ArtworkResolver {
    results_limit: u32,
}

impl ArtworkResolver {
    pub fn new(config: &Config) -> Self {
        Self {
            results_limit: config.apple_music.query_results_limit,
        }
    }

    pub fn resolve(&self, track: &TrackState) -> Result<String, String> {
        Err("not implemented".to_string())
    }
}
