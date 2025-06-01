mod error;
pub mod model;
// todo should use more high level api, pub to test here.
pub mod provider;
pub mod tool;

use error::LLMResult;
use model::{ChatMessage, ChatMessageDelta, ToolCall};
use provider::{LLMCallProcess, LLMProvider};
use tokio::sync::broadcast::Sender;
use tokio_stream::StreamExt;
use tool::ToolRegistry;
use tracing::{error, info};

use crate::core::stream_message::StreamMessage;

pub async fn chat(
    mut messages: Vec<ChatMessage>,
    stream: Sender<StreamMessage>,
    client: &impl LLMProvider,
    registry: &ToolRegistry,
) -> LLMResult<String> {
    let mut content = String::new();
    let mut tool_call = ToolCall::default();
    let mut current_process = LLMCallProcess::ChatStream;
    while current_process != LLMCallProcess::Finish {
        match current_process {
            LLMCallProcess::ChatStream => {
                current_process = LLMCallProcess::Finish; // default to finish
                let mut chat_stream = client.chat_stream(&messages).await?;
                while let Some(chunk) = chat_stream.next().await {
                    let chunk = chunk?;
                    match chunk.delta {
                        ChatMessageDelta::Content(s) => {
                            content.push_str(&s);
                            if s.is_empty() {
                                continue; // skip empty deltas
                            }
                            stream.send(StreamMessage::Delta(s))?;
                        }
                        ChatMessageDelta::ToolCalls(chunk) => {
                            if chunk.id.is_some() {
                                if let Some(name) = &chunk.function.name {
                                    stream
                                        .send(StreamMessage::Procedure(format!("Tools: {name}")))?;
                                }
                            }
                            current_process = LLMCallProcess::FunctionCall;
                            tool_call = tool_call.extend_chunk(chunk);
                        }
                    }
                }
            }
            LLMCallProcess::FunctionCall => {
                current_process = LLMCallProcess::Finish; // default to finish
                if let Some((f, _, _)) = registry.get(&tool_call.function.name) {
                    let r = f(serde_json::from_str(&tool_call.function.arguments)?);
                    messages.push(ChatMessage::assistant("").with_tool_call(tool_call.clone()));
                    info!("Tool call result: {:?}", r);
                    messages.push(ChatMessage::tool(
                        serde_json::to_string(&r)?,
                        tool_call.id.clone(),
                    ));
                    current_process = LLMCallProcess::ChatStream;
                    tool_call = ToolCall::default(); // reset tool call for next iteration
                } else {
                    error!(
                        "Tool function '{}' not found in registry",
                        &tool_call.function.name
                    );
                }
            }
            LLMCallProcess::Finish => {
                // No action needed, just exit the loop
            }
        }
    }
    Ok(content)
}
