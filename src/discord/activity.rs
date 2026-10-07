use std::sync::mpsc::Sender;

use serde_json::Value;

#[derive(Debug, Clone)]
pub enum ActivityPresence {
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
