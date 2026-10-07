use std::{
    collections::HashSet,
    env,
    io::Result,
    os::unix::net::UnixStream,
    path::PathBuf,
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

impl DiscordSocket {
    fn connect(client_id: &str) -> Result<Self> {
        for dir in possible_ipc_paths() {
            for index in 0..10 {
                let path = dir.join(format!("discord-ipc-{}", index));
                match UnixStream::connect(&path) {
                    Ok(stream) => {
                        stream.set_read_timeout(Some(Duration::from_secs(3)))?;
                        stream.set_write_timeout(Some(Duration::from_secs(3)))?;
                        let mut connection = Self { stream, nonce: 1 };
                        return Ok(connection);
                    }
                    Err(e) => return Err(e),
                }
            }
        }
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "error: no discord-ipc socket found",
        ));
    }
}

fn discord_thread(config: DiscordConfig, rx: Receiver<ActivityPresence>) {
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

fn possible_ipc_paths() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Some(value) = env::var_os("TMPDIR") {
        dirs.push(PathBuf::from(value));
    }

    dirs.push(PathBuf::from("/tmp"));

    dirs
}
