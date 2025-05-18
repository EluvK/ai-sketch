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

    use crate::llm::model::ChatMessageRole;

    #[allow(unused_imports)]
    use super::*;

    #[tokio::test]
    async fn test_llm_deepseek() {
        let client = deepseek::DeepSeekClient::new(
            std::env::var("DEEPSEEK_API_KEY").expect("DEEPSEEK_API_KEY not set"),
            "https://api.deepseek.com".to_string(),
            "deepseek-chat".to_string(),
        );

        // let client = openai::OpenAIClient::new(
        //     std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set"),
        //     "https://api.openai.com".to_string(),
        //     "gpt-4o-mini".to_string(),
        // );
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
