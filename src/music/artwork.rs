use std::{ops::Mul, time::Duration};

use serde::Deserialize;
use ureq::Agent;

use crate::{config::Config, music::bridge::TrackState};

#[derive(Debug, Clone, Deserialize)]
struct SearchResponse {
    results: Vec<SearchResult>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchResult {
    track_name: Option<String>,
    artist_name: Option<String>,
    collection_name: Option<String>,
    track_view_url: Option<String>,
    artwork_url100: Option<String>,
    track_time_millis: Option<u64>,
}

pub struct ArtworkResolver {
    agent: Agent,
    user_agent: String,
    resolution: u32,
    results_limit: u32,
}

impl ArtworkResolver {
    pub fn new(config: &Config) -> Self {
        Self {
            agent: Agent::config_builder()
                .timeout_global(Some(Duration::from_secs(
                    config.apple_music.timeout_seconds,
                )))
                .build()
                .new_agent(),
            user_agent: concat!("apple-music-rpc/", env!("CARGO_PKG_VERSION")).to_string(),
            resolution: config.apple_music.artwork_resolution,
            results_limit: config.apple_music.query_results_limit,
        }
    }

    pub fn resolve(&self, track: &TrackState) -> Result<String, String> {
        let cleaned_track_name = clean_string(&track.name);
        let squished_track_name = squished_string(&track.name);
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

        let exact_match = body
            .results
            .into_iter()
            // .filter(|result| result.artwork_url100.is_some())
            .find(|result| {
                let name_match = result
                    .track_name
                    .as_ref()
                    .map(|name| {
                        name.eq_ignore_ascii_case(&track.name)
                            || name.eq_ignore_ascii_case(&cleaned_track_name)
                            || name.eq_ignore_ascii_case(&squished_track_name)
                    })
                    .unwrap_or(false);
                let artist_match = result
                    .artist_name
                    .as_ref()
                    .map(|artist| artist.eq_ignore_ascii_case(&track.artist))
                    .unwrap_or(false);
                let duration_match = result
                    .track_time_millis
                    .as_ref()
                    .map(|time_millis| {
                        time_millis.abs_diff(track.duration.mul(1000 as f64) as u64) <= 3000
                    })
                    .unwrap_or(false);

                name_match && artist_match && duration_match
            });

        let artwork_url = exact_match
            .and_then(|result| result.artwork_url100)
            .ok_or_else(|| "error: failed to resolve artwork for the current track".to_string())?;
        let artwork_url = resize_artwork_url(&artwork_url, self.resolution);

        Ok(artwork_url)
    }
}

fn resize_artwork_url(url: &str, size: u32) -> String {
    let size_str = format!("{size}x{size}");
    url.replace("100x100bb.jpg", &format!("{size_str}bb.jpg"))
        .replace("100x100bb.png", &format!("{size_str}bb.png"))
}

fn clean_string(text: &str) -> String {
    let mut result = String::new();
    let mut nested_level = 0;
    for c in text.chars() {
        match c {
            '(' => nested_level += 1,
            ')' if nested_level > 0 => nested_level -= 1,
            _ if nested_level == 0 => result.push(c),
            _ => {}
        }
    }
    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn squished_string(name: &str) -> String {
    name.split_whitespace().collect::<Vec<_>>().join("")
}
