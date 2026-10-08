use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Map, Value, json};

use crate::{config::ActivityConfig, music::bridge::TrackState};

#[derive(Debug, Clone)]
pub enum ActivityPresence {
    Set(Value),
    Empty,
}

pub fn json_activity(config: &ActivityConfig, track: &TrackState, url: Option<String>) -> Value {
    let mut activity: Map<String, Value> = Map::new();
    activity.insert("type".to_string(), json!(2));
    activity.insert("status_display_type".to_string(), json!(2));
    activity.insert(
        "details".to_string(),
        json!(process_template(&config.details_format, &track)),
    );
    activity.insert(
        "state".to_string(),
        json!(process_template(&config.state_format, &track)),
    );

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();
    let position = track.progress.clamp(0.0, track.duration);
    let start = (now - position).floor() as i64;
    let end = (now - position + track.duration).floor() as i64;

    activity.insert(
        "timestamps".to_string(),
        json!({
            "start": start,
            "end": end
        }),
    );

    match url {
        Some(url) if !url.is_empty() => {
            activity.insert(
                "assets".to_string(),
                json!({
                    "large_image": url,
                    "large_text": process_template(&config.large_text_format, &track),
                }),
            );
        }
        _ => {}
    }

    Value::Object(activity)
}

fn process_template(template: &str, track: &TrackState) -> String {
    template
        .replace("{title}", &track.name)
        .replace("{artist}", &track.artist)
        .replace("{album}", &track.album)
}
