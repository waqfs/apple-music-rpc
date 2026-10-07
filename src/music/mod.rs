use std::{
    eprintln,
    sync::{Arc, atomic::AtomicI64},
    thread,
};

use crate::{artwork::ArtworkResolver, config::Config, music::bridge::AppleMusicBridge};

pub mod bridge;

const NO_TRACK_ID: i64 = i64::MIN;

pub fn spawn_music_thread(config: Config) {
    thread::Builder::new()
        .name("music-notifier".to_string())
        .spawn(move || music_thread(config))
        .expect("error: failed to spawn music-notifier thread");
}

fn music_thread(config: Config) {
    let bridge = match AppleMusicBridge::new() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: failed to initialize AppleMusicBridge: {e}");
            return;
        }
    };
    let artwork_resolver = ArtworkResolver::new(&config);

    let current_track_id = Arc::new(AtomicI64::new(NO_TRACK_ID));
    let mut last_track_id: Option<i64> = None;
}
