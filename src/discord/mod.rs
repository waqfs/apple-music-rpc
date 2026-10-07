use std::{
    sync::mpsc::{Receiver, RecvTimeoutError},
    time::Duration,
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
    loop {
        if connection.is_none() {
            // connect
        }

        let wait: Duration = if connection.is_some() {
            ping_delay
        } else {
            retry_delay
        };

        match rx.recv_timeout(wait) {
            Ok(activity) => {}
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return,
        }
    }
}
