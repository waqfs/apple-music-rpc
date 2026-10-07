use std::{
    sync::mpsc::{Receiver, RecvTimeoutError},
    time::{Duration, Instant},
};

use crate::{
    config::DiscordConfig,
    discord::{activity::ActivityPresence, ipc::DiscordSocket},
};

mod activity;
mod ipc;

pub fn discord_thread(config: DiscordConfig, rx: Receiver<ActivityPresence>) {
    let mut connection: Option<DiscordSocket> = None;
    let ping_delay = Duration::from_secs(config.ping_interval_seconds);
    let retry_delay = Duration::from_secs(config.reconnect_interval_seconds);
    let mut activity = ActivityPresence::Empty;
    let mut last_ping = Instant::now();

    loop {
        if connection.is_none() {
            match DiscordSocket::connect(&config.client_id) {
                Ok(mut connection2) => {
                    if let Err(e) = connection2.presence(&activity) {
                        eprintln!("error: failed to set discord presence: {e}");
                    } else {
                        connection = Some(connection2);
                        last_ping = Instant::now();
                    }
                }
                Err(_) => {}
            }
        }

        let wait: Duration = if connection.is_some() {
            ping_delay.saturating_sub(last_ping.elapsed())
        } else {
            retry_delay
        };

        match rx.recv_timeout(wait) {
            Ok(next_activity) => {
                activity = next_activity;
                if let Some(connection2) = connection.as_mut() {
                    if let Err(e) = connection2.presence(&activity) {
                        eprintln!("error: failed to set discord presence: {e}");
                        connection = None;
                    }
                }
            }
            Err(RecvTimeoutError::Timeout) => {
                if let Some(connection2) = connection.as_mut() {
                    if let Err(e) = connection2.ping() {
                        eprintln!("error: failed to ping discord ipc: {e}");
                        connection = None;
                    }
                    last_ping = Instant::now();
                }
            }
            Err(RecvTimeoutError::Disconnected) => return,
        }
    }
}
