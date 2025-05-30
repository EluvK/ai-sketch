use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub content: String,
    pub role: ChatMessageRole,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatMessageRole {
    User,
    Assistant,
    System,
    Tool, // tool calls result
}

// Represent the final response that will be returned && saved
// mock the details from different providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessageResponse {
    pub id: String,
    // pub object: String,
    pub message: String,
    pub created: i64,
    pub model: String,
    pub finish_reason: FinishReason,
    pub total_tokens: i64,
}

// Streamed response from the provider
// mock the details from different providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessageChunk {
    pub id: String,

    //? to be deleted, use `delta` instead
    pub delta_content: String, // usually a single token at choices[0].delta.content

    pub delta: ChatMessageDelta,
    pub created: i64,
    pub model: String,
    pub finish_reason: Option<FinishReason>,
    pub total_tokens: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    Stop,
    Length,
    ContentFilter,
    ToolCalls,
    InsufficientSystemResource,
    // User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChatMessageDelta {
    Content(String),       // The content of the message
    ToolCallsFunc(String), // Tool calls function in the message
    ToolCallsArgs(String), // Tool calls in the message
}
