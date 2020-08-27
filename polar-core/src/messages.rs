use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageKind {
    Print,
    Warning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub kind: MessageKind,
    pub msg: String,
}

#[derive(Clone, Debug)]
pub struct MessageQueue {
    messages: Arc<Mutex<VecDeque<Message>>>,
}

impl MessageQueue {
    pub fn new() -> Self {
        Self {
            messages: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn next(&self) -> Option<Message> {
        if let Ok(mut messages) = self.messages.lock() {
            messages.pop_front()
        } else {
            None
        }
    }

    pub fn push(&self, kind: MessageKind, msg: String) {
        let mut messages = self.messages.lock().unwrap();
        messages.push_back(Message { kind, msg });
    }

    pub fn extend<T: IntoIterator<Item = Message>>(&self, iter: T) {
        let mut messages = self.messages.lock().unwrap();
        messages.extend(iter)
    }
}

impl Default for MessageQueue {
    fn default() -> Self {
        Self::new()
    }
}
