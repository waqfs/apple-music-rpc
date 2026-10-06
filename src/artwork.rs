use std::println;

use ureq::Agent;

use crate::{config::Config, music::TrackState};

pub struct ArtworkResolver {
    agent: Agent,
    user_agent: String,
    results_limit: u32,
}

impl ArtworkResolver {
    pub fn new(config: &Config) -> Self {
        Self {
            agent: Agent::new_with_defaults(),
            user_agent: concat!("apple-music-rpc/", env!("CARGO_PKG_VERSION")).to_string(),
            results_limit: config.apple_music.query_results_limit,
        }
    }

    pub fn resolve(&self, track: &TrackState) -> Result<String, String> {
        let search_query = format!("{} {}", track.name, track.artist);
        let mut response = self
            .agent
            .get("https://itunes.apple.com/search")
            .query("term", search_query)
            .query("media", "music")
            .query("entity", "song")
            .query("limit", &self.results_limit.to_string())
            .header("User-Agent", &self.user_agent)
            .call()
            .map_err(|e| format!("error: artwork request failed: {e}"))?;

        println!("response: {:?}", response);

        Err("not implemented".to_string())
    }
}
