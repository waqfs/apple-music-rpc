use crate::{config::Config, music::TrackState};

pub struct ArtworkResolver {}

impl ArtworkResolver {
    pub fn new(config: &Config) -> Self {
        Self {}
    }

    pub fn resolve(&self, track: &TrackState) -> Result<String, String> {
        Err("not implemented".to_string())
    }
}
