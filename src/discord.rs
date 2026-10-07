use std::{
    env,
    io::{
        Error,
        ErrorKind::{InvalidData, InvalidInput, NotFound},
        Read, Result, Write,
    },
    os::unix::net::UnixStream,
    path::PathBuf,
    sync::mpsc::{Receiver, RecvTimeoutError, Sender},
    time::Duration,
};

use serde_json::Value;

use crate::config::DiscordConfig;

const MAX_FRAME_SIZE: usize = 65_536;

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
        return Err(Error::new(NotFound, "error: no discord-ipc socket found"));
    }

    fn write_json(&mut self, op: u32, payload: &Value) -> Result<()> {
        let payload = serde_json::to_vec(payload).map_err(|e| Error::new(InvalidData, e))?;
        if payload.len() > MAX_FRAME_SIZE {
            return Err(Error::new(
                InvalidInput,
                "error: discord ipc payload too large",
            ));
        }

        let mut frame = Vec::with_capacity(8 + payload.len());
        frame.extend_from_slice(&op.to_le_bytes());
        frame.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        frame.extend_from_slice(&payload);
        self.stream.write_all(&frame)
    }

    fn read_frame(&mut self) -> Result<(u32, Value)> {
        let mut header = [0u8; 8];
        self.stream.read_exact(&mut header)?;
        let op = u32::from_le_bytes(header[0..4].try_into().unwrap());
        let len = u32::from_le_bytes(header[4..8].try_into().unwrap()) as usize;
        if len > MAX_FRAME_SIZE {
            return Err(Error::new(
                InvalidData,
                "error: discord ipc frame too large",
            ));
        }

        let mut payload = vec![0u8; len];
        self.stream.read_exact(&mut payload)?;
        let value = if payload.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&payload).map_err(|e| Error::new(InvalidData, e))?
        };
        Ok((op, value))
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
