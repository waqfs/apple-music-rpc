use std::{
    env,
    io::{
        Error,
        ErrorKind::{ConnectionAborted, InvalidData, InvalidInput, NotFound},
        Read, Result, Write,
    },
    os::unix::net::UnixStream,
    path::PathBuf,
    process,
    time::Duration,
};

use serde_json::{Value, json};

use crate::discord::activity::ActivityPresence;

const MAX_FRAME_SIZE: usize = 65_536;
const OP_HANDSHAKE: u32 = 0;
const OP_FRAME: u32 = 1;
const OP_CLOSE: u32 = 2;
const OP_PING: u32 = 3;
const OP_PONG: u32 = 4;

pub struct DiscordSocket {
    stream: UnixStream,
    nonce: u64,
}

impl DiscordSocket {
    pub fn connect(client_id: &str) -> Result<Self> {
        for dir in possible_ipc_paths() {
            for index in 0..10 {
                let path = dir.join(format!("discord-ipc-{}", index));
                match UnixStream::connect(&path) {
                    Ok(stream) => {
                        stream.set_read_timeout(Some(Duration::from_secs(3)))?;
                        stream.set_write_timeout(Some(Duration::from_secs(3)))?;
                        let mut connection = Self { stream, nonce: 1 };
                        connection.handshake(client_id)?;
                        return Ok(connection);
                    }
                    Err(e) => return Err(e),
                }
            }
        }
        return Err(Error::new(NotFound, "error: no discord-ipc socket found"));
    }

    fn ping(&mut self) -> Result<()> {
        let payload = json!({ "nonce": &self.next_nonce() });
        self.write_json(OP_PING, &payload)?;

        loop {
            let (op, response) = self.read_frame()?;
            match op {
                OP_PONG => return Ok(()),
                OP_PING => self.write_json(OP_PONG, &response)?,
                OP_CLOSE => {
                    return Err(Error::new(
                        ConnectionAborted,
                        format!("error: discord ipc closed: {response}"),
                    ));
                }
                OP_FRAME => {
                    if response.get("evt").and_then(Value::as_str) == Some("ERROR") {
                        return Err(Error::new(
                            InvalidData,
                            format!("error: discord ipc error: {response}"),
                        ));
                    }
                }
                _ => {}
            }
        }
    }

    fn handshake(&mut self, client_id: &str) -> Result<()> {
        self.write_json(OP_HANDSHAKE, &json!({"v": 1, "client_id": client_id}))?;

        loop {
            let (op, response) = self.read_frame()?;
            match op {
                OP_PING => self.write_json(OP_PONG, &response)?,
                OP_CLOSE => {
                    return Err(Error::new(
                        ConnectionAborted,
                        format!("error: discord ipc closed: {response}"),
                    ));
                }
                OP_FRAME => {
                    let evt = response.get("evt").and_then(Value::as_str);
                    if response.get("cmd").and_then(Value::as_str) == Some("DISPATCH")
                        && evt == Some("READY")
                    {
                        return Ok(());
                    }
                    if evt == Some("ERROR") {
                        return Err(Error::new(
                            InvalidData,
                            format!("error: discord ipc error: {response}"),
                        ));
                    }
                }
                _ => {}
            }
        }
    }

    pub fn presence(&mut self, presence: &ActivityPresence) -> Result<()> {
        let nonce = self.next_nonce();
        let activity = match presence {
            ActivityPresence::Empty => Value::Null,
            ActivityPresence::Set(activity) => activity.clone(),
        };

        self.write_json(
            OP_FRAME,
            &json!({
                "cmd": "SET_ACTIVITY",
                "args": {
                    "pid": &process::id(),
                    "activity": activity,
                },
                "nonce": nonce
            }),
        )?;

        self.read_until_nonce(&nonce)
    }

    fn next_nonce(&mut self) -> String {
        let nonce = self.nonce;
        self.nonce = self.nonce.wrapping_add(1);
        nonce.to_string()
    }

    fn read_until_nonce(&mut self, nonce: &str) -> Result<()> {
        loop {
            let (op, payload) = self.read_frame()?;
            match op {
                OP_FRAME => {
                    if payload.get("evt").and_then(Value::as_str) == Some("ERROR") {
                        return Err(Error::new(
                            InvalidData,
                            format!("error: discord ipc error: {payload}"),
                        ));
                    }
                    if payload.get("nonce").and_then(Value::as_str) == Some(nonce) {
                        return Ok(());
                    }
                }
                OP_PING => self.write_json(OP_PONG, &payload)?,
                OP_CLOSE => {
                    return Err(Error::new(
                        ConnectionAborted,
                        format!("error: discord ipc closed: {payload}"),
                    ));
                }
                _ => {}
            }
        }
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

fn possible_ipc_paths() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Some(value) = env::var_os("TMPDIR") {
        dirs.push(PathBuf::from(value));
    }

    dirs.push(PathBuf::from("/tmp"));

    dirs
}
