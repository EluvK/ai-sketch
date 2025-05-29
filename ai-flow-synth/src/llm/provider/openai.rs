#![allow(dead_code)]
use std::pin::Pin;

use futures::{Stream, StreamExt};
use reqwest_eventsource::RequestBuilderExt;
use serde::Deserialize;

use crate::llm::{
    error::{LLMError, LLMResult},
    model::{ChatMessage, ChatMessageChunk, ChatMessageDelta, ChatMessageRole, FinishReason},
};

use super::LLMProvider;

pub struct OpenAIClient {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    model: String,
    tools: Vec<serde_json::Value>,
    // maybe other fields...
}

impl OpenAIClient {
    pub fn new(api_key: String, base_url: String, model: String) -> Self {
        let client = reqwest::Client::new();
        OpenAIClient {
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
}

impl Default for OpenAIClient {
    fn default() -> Self {
        Self::new(
            std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set"),
            "https://api.openai.com".to_string(),
            "gpt-4o-mini".to_string(),
        )
    }
}

#[async_trait::async_trait]
impl LLMProvider for OpenAIClient {
    async fn chat_stream(
        &self,
        messages: &[ChatMessage],
    ) -> LLMResult<Pin<Box<dyn Stream<Item = LLMResult<ChatMessageChunk>> + Send>>> {
        // Make the request to the OpenAI API
        let response = self
            .client
            .post(format!("{}/v1/chat/completions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&serde_json::json!(
                {
                    "model": self.model,
                    "messages": messages,
                    "stream": true,
                    "tools": self.tools,
                }
            ))
            .eventsource()?;
        let stream = async_stream::stream!({
            let mut response = response;
            while let Some(event) = response.next().await {
                let event = event
                    .map_err(|err| LLMError::LLMProvider(format!("OpenAI API error: {}", err)))?;
                let chunk: ChatMessageChunk = match event {
                    reqwest_eventsource::Event::Open => {
                        continue; // Open event, we can ignore it
                    }
                    reqwest_eventsource::Event::Message(event) => {
                        let data = event.data;
                        tracing::info!("Receive OpenAI API chunk: {}", data);
                        if data.trim() == "[DONE]" {
                            tracing::info!("OpenAI API stream DONE");
                            break;
                        } else if let Ok(chunk) = serde_json::from_str::<OpenAIChunkResp>(&data) {
                            chunk.into()
                        } else {
                            tracing::error!("OpenAI API response is not valid JSON: {}", data);
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
struct OpenAIChunkResp {
    id: String,
    object: String, // "chat.completion.chunk"
    created: i64,
    model: String,
    choices: Vec<OpenAIChunkChoice>,
    finish_reason: Option<FinishReason>,
}

#[derive(Debug, Clone, Deserialize)]
struct OpenAIChunkChoice {
    index: usize,
    delta: OpenAIChunkDelta,
    finish_reason: Option<FinishReason>,
}

#[derive(Debug, Clone, Deserialize)]
struct OpenAIChunkDelta {
    content: Option<String>, // tool_call场景可能是 None
    role: Option<ChatMessageRole>,
    tool_calls: Option<Vec<OpenAIToolCall>>,
}

#[derive(Debug, Clone, Deserialize)]
struct OpenAIToolCall {
    id: Option<String>, // 后续可能是 None
    index: i64,
    // r#type: Option<String>, // 目前一定是"function", 不重要了
    function: OpenAIToolFunction,
}

#[derive(Debug, Clone, Deserialize)]
struct OpenAIToolFunction {
    name: Option<String>,      // 后续可能是 None
    arguments: Option<String>, // 可能是 None
}

impl From<OpenAIChunkResp> for ChatMessageChunk {
    fn from(resp: OpenAIChunkResp) -> Self {
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
            finish_reason: resp.finish_reason,
            total_tokens: None, // OpenAI does not provide total_tokens in the stream response
        }
    }
}
