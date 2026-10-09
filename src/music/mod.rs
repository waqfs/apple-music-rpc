use std::{
    eprintln,
    sync::{
        Arc,
        atomic::{
            AtomicI64,
            Ordering::{Acquire, Release},
        },
        mpsc::Receiver,
    },
    thread,
    time::Instant,
};

use crate::{
    config::{ActivityConfig, Config},
    discord::{DiscordIPC, activity::json_activity},
    music::{
        artwork::ArtworkResolver,
        bridge::{AppleMusicBridge, PlaybackState, TrackState},
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

                    let activity = json_activity(&config.activity, &track, None);
                    discord.set_activity(activity);

                    if last_track_id == Some(track.local_id) {
                        continue;
                    }

                    spawn_artwork_thread(
                        artwork_resolver.clone(),
                        track.clone(),
                        config.activity.clone(),
                        discord.clone(),
                        Arc::clone(&current_track_id),
                    );

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

fn spawn_artwork_thread(
    resolver: ArtworkResolver,
    mut track: TrackState,
    config: ActivityConfig,
    discord: DiscordIPC,
    current_track_id: Arc<AtomicI64>,
) {
    thread::spawn(move || {
        let start = Instant::now();
        match resolver.resolve(&track) {
            Ok(details) => {
                if current_track_id.load(Acquire) != track.local_id {
                    return;
                }
                track.progress += start.elapsed().as_secs_f64();
                if track.duration > 0.0 {
                    track.progress = track.progress.min(track.duration);
                }
                discord.set_activity(json_activity(&config, &track, details.url));
            }
            Err(e) => {
                eprintln!(
                    "error: failed to resolve artwork for track {}: {e}",
                    track.name
                );
            }
        }
    });
}
