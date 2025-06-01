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
