#![allow(dead_code)]

use std::pin::Pin;

use futures::{Stream, StreamExt};
use reqwest_eventsource::{EventSource, RequestBuilderExt};
use serde::Deserialize;
use tracing::instrument;

use crate::llm::{
    error::{LLMError, LLMResult},
    model::{ChatMessage, ChatMessageDelta, ChatMessageRole, ChunkToolCall, FinishReason},
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

    async fn debug_chat(&self, messages: &[ChatMessage]) -> anyhow::Result<String> {
        let resp = self
            .client
            .post(format!("{}/v1/chat/completions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&serde_json::json!(
                {
                    "model": self.model,
                    "messages": messages,
                    "stream": false,
                    "tools": self.tools,
                }
            ))
            .send()
            .await?;
        resp.text().await.map_err(|e| {
            tracing::error!("Failed to get response text: {}", e);
            e.into()
        })
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
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<ChunkToolCall>>,
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
            (_, Some(tool_calls)) => match tool_calls.iter().next() {
                Some(tool_call) => ChatMessageDelta::ToolCalls(tool_call.clone()),
                None => {
                    tracing::warn!("No tool calls found in the response");
                    ChatMessageDelta::Content(String::new())
                }
            },
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

#[cfg(test)]
mod tests {
    use serde::Serialize;

    use crate::llm::tool::ToolRegistry;

    #[allow(unused_imports)]
    use super::*;

    #[derive(Serialize, Deserialize, schemars::JsonSchema)]
    pub struct GetWeatherParams {
        pub location: String,
    }

    fn get_weather(params: GetWeatherParams) -> serde_json::Value {
        serde_json::Value::String(format!("The weather in {} is sunny.", params.location))
    }

    #[tokio::test]
    async fn test_client() {
        let log_config = crate::utils::LogConfig::default();
        let _g = crate::utils::enable_log(&log_config).unwrap();
        let mut registry = ToolRegistry::new();
        registry.register::<GetWeatherParams, _>("get_weather", "获取天气", get_weather);

        let mut client = DeepSeekClient::default();
        client.add_tools(registry.export_all_tools());
        let messages = vec![
            ChatMessage::system("You are a helpful assistant."),
            ChatMessage::user(
                "Hi, would you please tell me what the time is it now, and weather in HangZhou",
            ),
        ];

        let resp = client
            .debug_chat(&messages)
            .await
            .expect("Failed to get response from DeepSeek API");
        tracing::info!("DeepSeek API response: {}", resp);
    }
}
