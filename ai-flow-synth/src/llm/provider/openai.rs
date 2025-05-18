#![allow(dead_code)]
use std::pin::Pin;

use futures::{Stream, StreamExt};
use reqwest_eventsource::RequestBuilderExt;
use serde::Deserialize;

use crate::llm::{
    error::{LLMError, LLMResult},
    model::{ChatMessage, ChatMessageChunk, FinishReason},
};

use super::LLMProvider;

pub struct OpenAIClient {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    model: String,
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
        }
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
                            match serde_json::from_str::<OpenAIChunkResp>(&data) {
                                Ok(chunk) => {
                                    yield Ok(chunk.into());
                                }
                                Err(err) => {
                                    yield Err(LLMError::from(err));
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
    content: Option<String>,
}

impl From<OpenAIChunkResp> for ChatMessageChunk {
    fn from(chunk: OpenAIChunkResp) -> Self {
        ChatMessageChunk {
            id: chunk.id,
            delta_content: chunk.choices[0].delta.content.clone().unwrap_or_default(),
            created: chunk.created,
            model: chunk.model,
            finish_reason: chunk.finish_reason,
            total_tokens: None, // OpenAI does not provide total_tokens in the stream response
        }
    }
}
