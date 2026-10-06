use objc2::{
    msg_send,
    rc::Retained,
    runtime::{AnyClass, AnyObject},
};
use objc2_foundation::NSString;
use serde::Serialize;

#[link(name = "ScriptingBridge", kind = "framework")]
unsafe extern "C" {}

const MUSIC_BUNDLE_ID: &str = "com.apple.Music";
const PLAYER_STOPPED: u32 = 0x6b50_5353; // kPSS
const PLAYER_PLAYING: u32 = 0x6b50_5350; // kPSP
const PLAYER_PAUSED: u32 = 0x6b50_5370; // kPSp
const PLAYER_FAST_FORWARDING: u32 = 0x6b50_5346; // kPSF
const PLAYER_REWINDING: u32 = 0x6b50_5352; // kPSR

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum PlaybackState {
    Playing,
    Paused,
    Stopped,
}

#[derive(Debug, Clone, Serialize)]
pub struct MusicState {
    pub state: PlaybackState,
    pub track: Option<TrackState>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TrackState {
    pub name: String,
    pub artist: String,
    pub album: String,
    pub duration: f64,
    pub progress: f64,
}

pub struct AppleMusicBridge {
    app: Retained<AnyObject>,
}

impl AppleMusicBridge {
    pub fn new() -> Result<Self, String> {
        let clazz = AnyClass::get(c"SBApplication")
            .ok_or_else(|| "error: SBApplication class is not available".to_string())?;

        let bundle_id = NSString::from_str(MUSIC_BUNDLE_ID);
        let app: Option<Retained<AnyObject>> =
            unsafe { msg_send![clazz, applicationWithBundleIdentifier: &*bundle_id] };
        let app = app.ok_or_else(|| "error: failed to bridge the Apple Music app".to_string())?;
        Ok(Self { app })
    }

    pub fn get_state(&self) -> Result<MusicState, String> {
        if !self.is_running() {
            return Ok(MusicState {
                state: PlaybackState::Stopped,
                track: None,
            });
        }

        let state = match self.get_player_state() {
            PLAYER_PLAYING | PLAYER_FAST_FORWARDING | PLAYER_REWINDING => PlaybackState::Playing,
            PLAYER_PAUSED => PlaybackState::Paused,
            PLAYER_STOPPED => PlaybackState::Stopped,
            _ => PlaybackState::Stopped,
        };

        if state != PlaybackState::Playing {
            return Ok(MusicState { state, track: None });
        }

        Err("not implemented".to_string())
    }

    pub fn is_running(&self) -> bool {
        unsafe { msg_send![&*self.app, isRunning] }
    }

    pub fn get_player_state(&self) -> u32 {
        unsafe { msg_send![&*self.app, playerState] }
    }

    pub fn get_current_track(&self) -> Result<Retained<AnyObject>, String> {
        let track: Option<Retained<AnyObject>> = unsafe { msg_send![&*self.app, currentTrack] };
        track.ok_or_else(|| "error: failed to get the current track".to_string())
    }

    pub fn get_current_track_name(&self, track: Retained<AnyObject>) -> Result<String, String> {
        let name: Option<Retained<NSString>> = unsafe { msg_send![&*track, name] };
        name.map(|n| n.to_string())
            .ok_or_else(|| "error: failed to get the current track name".to_string())
    }

    pub fn get_current_track_artist(&self, track: Retained<AnyObject>) -> Result<String, String> {
        let artist: Option<Retained<NSString>> = unsafe { msg_send![&*track, artist] };
        artist
            .map(|a| a.to_string())
            .ok_or_else(|| "error: failed to get the current track artist".to_string())
    }

    pub fn get_current_track_album(&self, track: Retained<AnyObject>) -> Result<String, String> {
        let album: Option<Retained<NSString>> = unsafe { msg_send![&*track, album] };
        album
            .map(|a| a.to_string())
            .ok_or_else(|| "error: failed to get the current track album".to_string())
    }

    pub fn get_current_track_duration(&self, track: Retained<AnyObject>) -> Result<f64, String> {
        let duration: f64 = unsafe { msg_send![&*track, duration] };
        if duration.is_nan() {
            Err("error: failed to get the current track duration".to_string())
        } else {
            Ok(duration)
        }
    }

    pub fn get_current_track_progress(&self) -> Result<f64, String> {
        let progress: f64 = unsafe { msg_send![&*self.app, playerPosition] };
        if progress.is_nan() {
            Err("error: failed to get the current track progress".to_string())
        } else {
            Ok(progress)
        }
    }
}
