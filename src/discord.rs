use std::{
    os::unix::net::UnixStream,
    sync::mpsc::{Receiver, RecvTimeoutError, Sender},
    time::Duration,
};

use toml::Value;

use crate::config::DiscordConfig;

#[derive(Debug, Clone)]
enum ActivityPresence {
    Set(Value),
    Empty,
}

pub struct DiscordActivity {
    tx: Sender<ActivityPresence>,
}

impl DiscordActivity {
    pub fn set_activity(&self, activity: Value) {
        self.tx.send(ActivityPresence::Set(activity));
    }

    pub fn clear_activity(&self) {
        self.tx.send(ActivityPresence::Empty);
    }
}

struct DiscordSocket {
    stream: UnixStream,
    nonce: u64,
}

fn discord_thread(config: DiscordConfig, rx: Receiver<ActivityPresence>) {
    let mut connection: Option<DiscordSocket> = None;
    loop {
        if connection.is_none() {
            // connect
        }

        let wait: Duration = if connection.is_some() {
            // ping delay
            Duration::from_secs(1)
        } else {
            // retry delay
            Duration::from_secs(1)
        };

        match rx.recv_timeout(wait) {
            Ok(activity) => {}
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return,
        }
    }
}
