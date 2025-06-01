use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatMessageRole {
    User,
    Assistant,
    System,
    Tool, // tool calls result
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub content: String,
    pub role: ChatMessageRole,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>, // tool call id, if role is Tool, this is a tool call result message
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tool_calls: Vec<ToolCall>, // tool calls in the message
}

impl Default for ChatMessage {
    fn default() -> Self {
        ChatMessage {
            content: String::new(),
            role: ChatMessageRole::User,
            tool_call_id: None,
            tool_calls: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolCall {
    pub id: String,
    pub index: i64,
    pub r#type: String, // currently always "function"
    pub function: ToolFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolFunction {
    pub name: String,
    pub arguments: String, // JSON string
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkToolCall {
    pub id: Option<String>, // maybe None
    pub index: i64,
    pub r#type: Option<String>, // currently always "function"
    pub function: ChunkToolFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkToolFunction {
    pub name: Option<String>, // maybe None
    pub arguments: String,    // JSON string
}

impl From<ChunkToolCall> for ToolCall {
    fn from(chunk_tool_call: ChunkToolCall) -> Self {
        ToolCall {
            id: chunk_tool_call.id.unwrap_or_default(),
            index: chunk_tool_call.index,
            r#type: chunk_tool_call.r#type.unwrap_or("function".to_string()),
            function: ToolFunction {
                name: chunk_tool_call.function.name.unwrap_or_default(),
                arguments: chunk_tool_call.function.arguments,
            },
        }
    }
}

impl ToolCall {
    pub fn extend_chunk(mut self, chunk_tool_call: ChunkToolCall) -> Self {
        if let Some(id) = chunk_tool_call.id {
            self.id = id;
        }
        if let Some(r#type) = chunk_tool_call.r#type {
            self.r#type = r#type;
        }
        if let Some(name) = chunk_tool_call.function.name {
            self.function.name = name;
        }
        self.function
            .arguments
            .push_str(&chunk_tool_call.function.arguments);
        self
    }
}

impl ChatMessage {
    pub fn user(content: impl ToString) -> Self {
        ChatMessage {
            content: content.to_string(),
            ..Default::default()
        }
    }
    pub fn assistant(content: impl ToString) -> Self {
        ChatMessage {
            content: content.to_string(),
            role: ChatMessageRole::Assistant,
            ..Default::default()
        }
    }
    pub fn system(content: impl ToString) -> Self {
        ChatMessage {
            content: content.to_string(),
            role: ChatMessageRole::System,
            ..Default::default()
        }
    }
    pub fn tool(content: impl ToString, tool_call_id: String) -> Self {
        ChatMessage {
            content: content.to_string(),
            role: ChatMessageRole::Tool,
            tool_call_id: Some(tool_call_id),
            ..Default::default()
        }
    }
    pub fn with_tool_call(mut self, tool_call: ToolCall) -> Self {
        self.tool_calls.push(tool_call);
        self
    }
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChatMessageDelta {
    Content(String), // The content of the message
    ToolCalls(ChunkToolCall),
}
