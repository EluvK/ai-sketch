#![allow(dead_code)]

use std::pin::Pin;

use futures::{Stream, StreamExt};
use reqwest_eventsource::{EventSource, RequestBuilderExt};
use serde::Deserialize;
use tracing::instrument;
// use tokio::io::{AsyncBufReadExt, BufReader};
// use tokio_util::io::StreamReader;

use crate::llm::{
    error::{LLMError, LLMResult},
    model::{ChatMessage, ChatMessageDelta, ChatMessageRole, FinishReason},
};

use super::{ChatMessageChunk, LLMProvider};

pub struct DeepSeekClient {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    model: String,
    tools: Vec<serde_json::Value>,
    // maybe other fields...
}

impl Default for DeepSeekClient {
    fn default() -> Self {
        Self::new(
            std::env::var("DEEPSEEK_API_KEY").expect("DEEPSEEK_API_KEY not set"),
            "https://api.deepseek.com".to_string(),
            "deepseek-chat".to_string(),
        )
    }
}

impl DeepSeekClient {
    pub fn new(api_key: String, base_url: String, model: String) -> Self {
        let client = reqwest::Client::new();
        DeepSeekClient {
            client,
            api_key,
            base_url,
            model,
            tools: Vec::new(),
        }
    }

    pub fn add_tool(&mut self, tool: serde_json::Value) {
        self.tools.push(tool);
    }

    pub fn add_tools(&mut self, tools: Vec<serde_json::Value>) {
        self.tools.extend(tools);
    }

    fn client_chat_stream(&self, message: &[ChatMessage]) -> LLMResult<EventSource> {
        let resp = self
            .client
            .post(format!("{}/v1/chat/completions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&serde_json::json!(
                {
                    "model": self.model,
                    "messages": message,
                    "stream": true,
                    "tools": self.tools,
                }
            ))
            .eventsource()?;
        Ok(resp)
    }
}

#[async_trait::async_trait]
impl LLMProvider for DeepSeekClient {
    #[instrument(
        name = "DeepSeekClient::chat_stream",
        skip(self, messages),
        fields(
            model = %self.model,
            base_url = %self.base_url
        )
    )]
    async fn chat_stream(
        &self,
        messages: &[ChatMessage],
    ) -> LLMResult<Pin<Box<dyn Stream<Item = LLMResult<ChatMessageChunk>> + Send>>> {
        let mut event_source = self.client_chat_stream(messages)?;
        let stream = async_stream::stream!({
            while let Some(event) = event_source.next().await {
                let event = event
                    .map_err(|err| LLMError::LLMProvider(format!("DeepSeek API error: {}", err)))?;

                let chunk: ChatMessageChunk = match event {
                    reqwest_eventsource::Event::Open => {
                        continue; // Open event, we can ignore it
                    }
                    reqwest_eventsource::Event::Message(event) => {
                        let data = event.data;
                        tracing::info!("Received DeepSeek API chunk: {}", data);
                        if data.trim() == "[DONE]" {
                            tracing::info!("DeepSeek API stream DONE");
                            break;
                        } else if let Ok(chunk) = serde_json::from_str::<DeepSeekChunkResp>(&data) {
                            chunk.into()
                        } else {
                            tracing::error!("DeepSeek API response is not valid JSON: {}", data);
                            continue; // Skip this chunk
                        }
                    }
                };
                tracing::info!("Yielding chunk: {:?}", chunk);
                yield Ok(chunk);
            }
        });

        Ok(Box::pin(stream))
    }
}

#[derive(Debug, Clone, Deserialize)]
struct DeepSeekChunkResp {
    id: String,
    object: String, // chat.completion.chunk
    created: i64,
    model: String,
    choices: Vec<DeepSeekChoice>,
    usage: Option<DeepSeekUsage>,
}

#[derive(Debug, Clone, Deserialize)]
struct DeepSeekChoice {
    index: i64,
    delta: DeepSeekDelta,
    finish_reason: Option<FinishReason>, // 可能是 None
}

#[derive(Debug, Clone, Deserialize)]
struct DeepSeekDelta {
    content: Option<String>, // tool_call场景可能是 None
    role: Option<ChatMessageRole>,
    tool_calls: Option<Vec<DeepSeekToolCall>>,
}

#[derive(Debug, Clone, Deserialize)]
struct DeepSeekToolCall {
    id: Option<String>, // 后续可能是 None
    index: i64,
    // r#type: Option<String>, // 目前一定是"function", 不重要了
    function: DeepSeekFunction,
}

#[derive(Debug, Clone, Deserialize)]
struct DeepSeekFunction {
    name: Option<String>,      // 后续可能是 None
    arguments: Option<String>, // 可能是 None
}

#[derive(Debug, Clone, Deserialize)]
struct DeepSeekUsage {
    prompt_tokens: i64,
    completion_tokens: i64,
    total_tokens: i64,
}

impl From<DeepSeekChunkResp> for ChatMessageChunk {
    fn from(resp: DeepSeekChunkResp) -> Self {
        let delta = match (
            &resp.choices[0].delta.content,
            &resp.choices[0].delta.tool_calls,
        ) {
            (Some(content), _) => ChatMessageDelta::Content(content.to_owned()),
            (_, Some(tool_calls)) => {
                match tool_calls
                    .iter()
                    .next()
                    .map(|tool_call| (&tool_call.function.name, &tool_call.function.arguments))
                {
                    Some((Some(name), _)) => ChatMessageDelta::ToolCallsFunc(name.to_owned()),
                    Some((_, Some(args))) => ChatMessageDelta::ToolCallsArgs(args.to_owned()),
                    _ => ChatMessageDelta::Content(String::new()),
                }
            }
            _ => ChatMessageDelta::Content(String::new()),
        };
        ChatMessageChunk {
            id: resp.id,
            delta_content: resp.choices[0].delta.content.clone().unwrap_or_default(),
            delta,
            created: resp.created,
            model: resp.model,
            finish_reason: resp.choices[0].finish_reason.clone(),
            total_tokens: resp.usage.map(|u| u.total_tokens),
        }
    }
}
