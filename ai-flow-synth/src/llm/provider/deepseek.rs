#![allow(dead_code)]

use std::pin::Pin;

use futures::{Stream, StreamExt};
use reqwest_eventsource::RequestBuilderExt;
use serde::Deserialize;
// use tokio::io::{AsyncBufReadExt, BufReader};
// use tokio_util::io::StreamReader;

use crate::llm::{
    error::{LLMError, LLMResult},
    model::{ChatMessage, FinishReason},
};

use super::{ChatMessageChunk, LLMProvider};

pub struct DeepSeekClient {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    model: String,
    // maybe other fields...
}

impl DeepSeekClient {
    pub fn new(api_key: String, base_url: String, model: String) -> Self {
        let client = reqwest::Client::new();
        DeepSeekClient {
            client,
            api_key,
            base_url,
            model,
        }
    }
}

#[async_trait::async_trait]
impl LLMProvider for DeepSeekClient {
    async fn chat_stream(
        &self,
        messages: &[ChatMessage],
    ) -> LLMResult<Pin<Box<dyn Stream<Item = LLMResult<ChatMessageChunk>> + Send>>> {
        // Make the request to the DeepSeek API
        let response = self
            .client
            .post(format!("{}/v1/chat/completions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&serde_json::json!(
                {
                    "model": self.model,
                    "messages": messages,
                    "stream": true,
                }
            ))
            .eventsource()?;

        let stream = async_stream::stream! {
            let mut response = response;
            while let Some(event) = response.next().await {
                match event {
                    Ok(event) => {
                        if let reqwest_eventsource::Event::Message(message) = event {
                            let data = message.data;
                            if data.trim() == "[DONE]" {
                                break;
                            }
                            match serde_json::from_str::<DeepSeekChunkResp>(&data) {
                                Ok(chunk) => {
                                    yield Ok(chunk.into());
                                }
                                Err(err) => {
                                    // Handle the case where the payload is not valid JSON
                                    yield Err(LLMError::LLMProvider(format!(
                                        "DeepSeek API response is not valid JSON: {}",
                                        err
                                    )));
                                }
                            }
                        }
                    }
                    Err(reqwest_eventsource::Error::StreamEnded) => {
                        break;
                    }
                    // maybe handle other errors here like 401?
                    Err(err) => {
                        yield Err(LLMError::LLMProvider(format!(
                            "DeepSeek API error: {}",
                            err
                        )));
                    }
                }
            }
        };
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
    content: Option<String>, // 可能是 None
}

#[derive(Debug, Clone, Deserialize)]
struct DeepSeekUsage {
    prompt_tokens: i64,
    completion_tokens: i64,
    total_tokens: i64,
}

impl From<DeepSeekChunkResp> for ChatMessageChunk {
    fn from(resp: DeepSeekChunkResp) -> Self {
        ChatMessageChunk {
            id: resp.id,
            delta_content: resp.choices[0].delta.content.clone().unwrap_or_default(),
            created: resp.created,
            model: resp.model,
            finish_reason: resp.choices[0].finish_reason.clone(),
            total_tokens: resp.usage.map(|u| u.total_tokens),
        }
    }
}
