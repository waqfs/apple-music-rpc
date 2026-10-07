use std::{
    eprintln,
    sync::{
        Arc,
        atomic::{AtomicI64, Ordering::Release},
        mpsc::Receiver,
    },
    thread,
};

use crate::{
    config::Config,
    discord::{DiscordIPC, activity::json_activity},
    music::{
        artwork::ArtworkResolver,
        bridge::{AppleMusicBridge, PlaybackState},
    },
};

pub mod artwork;
pub mod bridge;
pub mod notification;

const NO_TRACK_ID: i64 = i64::MIN;

pub fn spawn_music_thread(config: Config, rx: Receiver<()>, discord: DiscordIPC) {
    thread::Builder::new()
        .name("music-notifier".to_string())
        .spawn(move || music_thread(config, rx, discord))
        .expect("error: failed to spawn music-notifier thread");
}

fn music_thread(config: Config, rx: Receiver<()>, discord: DiscordIPC) {
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

    while rx.recv().is_ok() {
        match bridge.get_state() {
            Ok(state) => match state.state {
                PlaybackState::Paused | PlaybackState::Stopped => {
                    current_track_id.store(NO_TRACK_ID, Release);
                    discord.clear_activity();
                }
                PlaybackState::Playing => {
                    let Some(track) = state.track else {
                        current_track_id.store(NO_TRACK_ID, Release);
                        discord.clear_activity();
                        continue;
                    };

                    current_track_id.store(track.local_id, Release);

                    let url = match artwork_resolver.resolve(&track) {
                        Ok(url) => Some(url),
                        Err(_) => None,
                    };

                    let activity = json_activity(&config.activity, &track, url);
                    discord.set_activity(activity);

                    last_track_id = Some(track.local_id);
                }
            },
            Err(e) => {
                eprintln!("error: failed to get music state: {e}");
                current_track_id.store(NO_TRACK_ID, Release);
                discord.clear_activity();
                continue;
            }
        }
    }
}
