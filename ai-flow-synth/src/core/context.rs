use std::collections::HashMap;

use serde_json::Value;
use tokio::sync::broadcast;

use super::stream_message::StreamMessage;

#[derive(Debug, Clone)]
pub struct Context {
    /// uuid for one context
    _id: String,
    /// context data, the nodes can set and get the data to communicate with each other
    data: HashMap<String, Value>,
    /// stream for the context, the nodes can send messages to the stream
    stream: broadcast::Sender<StreamMessage>, //? consider using a generic type here
}

// maybe we should define some reserved keys for the context
pub static CONTEXT_RESULT: &str = "result";
// pub static CONTEXT_ERROR: &str = "error";

impl Context {
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(100);
        Context {
            _id: uuid::Uuid::new_v4().to_string(),
            data: HashMap::new(),
            stream: tx,
        }
    }

    pub fn set(&mut self, key: &str, value: Value) {
        self.data.insert(key.to_owned(), value);
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.data.get(key)
    }

    pub fn remove(&mut self, key: &str) {
        self.data.remove(key);
    }

    pub fn stream(&mut self, _stream_id: &str) -> broadcast::Sender<StreamMessage> {
        self.stream.clone()
    }

    pub fn listen(&self) -> tokio_stream::wrappers::BroadcastStream<StreamMessage> {
        let receiver = self.stream.subscribe();
        tokio_stream::wrappers::BroadcastStream::new(receiver)
    }
}
