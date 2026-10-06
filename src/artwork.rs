use std::println;

use serde::Deserialize;
use ureq::Agent;

use crate::{config::Config, music::TrackState};

#[derive(Debug, Deserialize)]
struct SearchResponse {
    results: Vec<SearchResult>,
}

#[derive(Debug, Deserialize)]
struct SearchResult {
    track_name: Option<String>,
    artist_name: Option<String>,
    collection_name: Option<String>,
    track_view_url: Option<String>,
    artwork_uri100: Option<String>,
    track_time_millis: Option<u64>,
}

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

        let body: SearchResponse = response
            .body_mut()
            .read_json()
            .map_err(|e| format!("error: artwork request returned malformed json: {e}"))?;

        println!("search results: {:?}", body.results);

        Err("not implemented".to_string())
    }
}
