pub mod deepseek;
pub mod openai;

use super::{
    error::LLMResult,
    model::{ChatMessage, ChatMessageChunk},
};
use futures::Stream;
use std::pin::Pin;

#[async_trait::async_trait]
pub trait LLMProvider {
    async fn chat_stream(
        &self,
        messages: &[ChatMessage],
    ) -> LLMResult<Pin<Box<dyn Stream<Item = LLMResult<ChatMessageChunk>> + Send>>>;
}

#[cfg(test)]
mod tests {
    use futures::StreamExt;

    use crate::{llm::model::ChatMessageRole, utils::LogConfig};

    #[allow(unused_imports)]
    use super::*;

    #[tokio::test]
    async fn test_llm_deepseek() {
        let log_config = LogConfig::default();
        let _g = crate::utils::enable_log(&log_config).unwrap();
        let client = deepseek::DeepSeekClient::default();

        let messages = vec![
            ChatMessage {
                content: "You are a helpful assistant.".to_string(),
                role: ChatMessageRole::System,
            },
            ChatMessage {
                content: "Hi, would you please tell me a joke in 20 words".to_string(),
                role: ChatMessageRole::User,
            },
        ];
        // let mut stream = client.chat_stream(&messages).await.unwrap();
        let mut stream = client.chat_stream(&messages).await.unwrap();
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(chunk) => {
                    println!("Received message: {:?}", chunk);
                }
                Err(err) => {
                    eprintln!("Error: {:?}", err);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_llm_function_call() {
        let log_config = LogConfig::default();
        let _g = crate::utils::enable_log(&log_config).unwrap();
        let mut client = deepseek::DeepSeekClient::default();
        // let mut client = openai::OpenAIClient::default();
        client.add_tool(serde_json::json!({
            "type": "function",
            "function": {
                "name": "get_weather",
                "description": "Get weather of an location, the user should supply a location first",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "location": {
                            "type": "string",
                            "description": "The city and state, e.g. San Francisco, CA"
                        }
                    },
                    "required": ["location"]
                },
            }
        }));
        let messages = vec![
            ChatMessage {
                content: "You are a helpful assistant.".to_string(),
                role: ChatMessageRole::System,
            },
            ChatMessage {
                content:
                    "Hi, would you please tell me what the time is it now, and weather in HangZhou"
                        .to_string(),
                role: ChatMessageRole::User,
            },
        ];
        let mut stream = client.chat_stream(&messages).await.unwrap();
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(chunk) => {
                    println!("Received chunk: {:?}", chunk);
                }
                Err(err) => {
                    eprintln!("Error: {:?}", err);
                }
            }
        }
    }
}
