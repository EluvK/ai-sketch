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

#[derive(Debug, Clone, PartialEq)]
pub enum LLMCallProcess {
    ChatStream,
    FunctionCall,
    Finish,
}

#[cfg(test)]
mod tests {
    use futures::StreamExt;
    use serde::{Deserialize, Serialize};

    use crate::{
        llm::{
            model::{ChatMessageDelta, ChatMessageRole},
            tool::ToolRegistry,
        },
        utils::LogConfig,
    };

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

    #[derive(Serialize, Deserialize, schemars::JsonSchema)]
    pub struct GetWeatherParams {
        pub location: String,
    }

    fn get_weather(params: GetWeatherParams) -> serde_json::Value {
        serde_json::Value::String(format!("The weather in {} is sunny.", params.location))
    }

    #[tokio::test]
    async fn test_llm_function_call() -> anyhow::Result<()> {
        let log_config = LogConfig::default();
        let _g = crate::utils::enable_log(&log_config).unwrap();
        let mut client = deepseek::DeepSeekClient::default();

        let mut registry = ToolRegistry::new();
        registry.register::<GetWeatherParams, _>("get_weather", "获取天气", get_weather);

        // let mut client = openai::OpenAIClient::default();
        client.add_tools(registry.export_all_tools());
        let mut messages = vec![
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

        let mut current_process = LLMCallProcess::ChatStream;

        let mut content = String::new();
        let mut tool_call_fn = String::new();
        let mut tool_call_args = String::new();
        while current_process != LLMCallProcess::Finish {
            match current_process {
                LLMCallProcess::ChatStream => {
                    println!("Processing chat stream...");
                    current_process = LLMCallProcess::Finish;
                    let mut stream = client.chat_stream(&messages).await.unwrap();
                    while let Some(chunk) = stream.next().await {
                        let chunk = chunk?;
                        println!("Received chunk: {:?}", chunk);
                        match chunk.delta {
                            ChatMessageDelta::Content(s) => content.push_str(&s),
                            ChatMessageDelta::ToolCallsFunc(s) => {
                                current_process = LLMCallProcess::FunctionCall;
                                tool_call_fn = s;
                            }
                            ChatMessageDelta::ToolCallsArgs(s) => {
                                tool_call_args.push_str(&s);
                            }
                        }
                    }
                }
                LLMCallProcess::FunctionCall => {
                    println!("Processing function call...");
                    current_process = LLMCallProcess::Finish;
                    if let Some((f, _, _)) = registry.get(&tool_call_fn) {
                        let r = f(serde_json::from_str(&tool_call_args)?);
                        println!("Tool call result: {:?}", r);
                        messages.push(ChatMessage {
                            content: serde_json::to_string(&r)?,
                            role: ChatMessageRole::User,
                        });
                        current_process = LLMCallProcess::ChatStream;
                    } else {
                        eprintln!("Tool function '{}' not found in registry", tool_call_fn);
                    }
                    // Here we would normally handle the function call logic
                }
                LLMCallProcess::Finish => {
                    println!("Finished processing.");
                }
            }
        }

        Ok(())
    }
}
