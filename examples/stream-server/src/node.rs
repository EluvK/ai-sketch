use ai_flow_synth::{
    core::{
        context::Context,
        node::{Node, NodeResult},
        status::Status,
        stream_message::StreamMessage,
    },
    llm::{
        model::{ChatMessage, ChatMessageRole},
        provider::{self, LLMProvider},
    },
};
use anyhow::Result;
use serde_json::Value;

use tokio_stream::StreamExt;

#[derive(Debug, Clone, Default, PartialEq)]
pub enum JobStatus {
    #[default]
    NotStarted,
    Failed,
    Written,
    Finished,
}

impl Status for JobStatus {
    fn failed() -> Self {
        JobStatus::Failed
    }
}

pub struct WriterNode {
    prompt: String,
}

impl WriterNode {
    pub fn new(prompt: String) -> Self {
        WriterNode { prompt }
    }
}

#[async_trait::async_trait]
impl Node for WriterNode {
    type FlowStatus = JobStatus;

    async fn execute(&self, context: &mut Context) -> Result<Value> {
        println!("prompt: {}", self.prompt);
        let stream = context.stream("writer_stream");
        println!("writing...");
        let client = provider::deepseek::DeepSeekClient::new(
            std::env::var("DEEPSEEK_API_KEY").expect("DEEPSEEK_API_KEY not set"),
            "https://api.deepseek.com".to_string(),
            "deepseek-chat".to_string(),
        );

        let messages = vec![
            ChatMessage {
                content: "You are a professional writer.".to_string(),
                role: ChatMessageRole::System,
            },
            ChatMessage {
                content: self.prompt.clone(),
                role: ChatMessageRole::User,
            },
        ];
        let mut chat_stream = client.chat_stream(&messages).await?;
        let mut content = String::new();
        while let Some(chunk) = chat_stream.next().await {
            let chunk = chunk?;
            content.push_str(&chunk.delta_content);
            stream.send(StreamMessage::Delta(chunk.delta_content))?;
        }
        println!("Received content: {}", content);
        context.set("draft", serde_json::Value::String(content.clone()));

        Ok(serde_json::json!({ "status": "write" }))
    }
    async fn after_exec(
        &self,
        context: &mut Context,
        result: &Result<Value>,
    ) -> Result<NodeResult<Self::FlowStatus>> {
        println!("after_exec: {:?}", result);
        Ok(NodeResult {
            status: JobStatus::Written,
            message: "done".to_owned(),
        })
    }
}
pub struct EditorNode {
    prompt: String,
}

impl EditorNode {
    pub fn new(prompt: String) -> Self {
        EditorNode { prompt }
    }
}

#[async_trait::async_trait]
impl Node for EditorNode {
    type FlowStatus = JobStatus;

    async fn execute(&self, context: &mut Context) -> Result<Value> {
        // println!("prompt: {}", self.prompt);
        let stream = context.stream("editor_stream");
        let content = context.get("draft").unwrap_or(&serde_json::Value::Null);
        if content.is_null() {
            return Err(anyhow::anyhow!("draft not found"));
        }
        let content = content.as_str().unwrap_or("");
        println!("content: {}", content);
        println!("editing...");
        let client = provider::deepseek::DeepSeekClient::new(
            std::env::var("DEEPSEEK_API_KEY").expect("DEEPSEEK_API_KEY not set"),
            "https://api.deepseek.com".to_string(),
            "deepseek-chat".to_string(),
        );
        let messages = vec![
            ChatMessage {
                content: format!(
                    "You are a professional editor. Find the mistakes in the text and correct them. {} return the result text ONLY.",
                    self.prompt
                ),
                role: ChatMessageRole::System,
            },
            ChatMessage {
                content: content.to_string(),
                role: ChatMessageRole::User,
            },
        ];

        let mut chat_stream = client.chat_stream(&messages).await?;
        let mut content = String::new();
        while let Some(chunk) = chat_stream.next().await {
            let chunk = chunk?;
            content.push_str(&chunk.delta_content);
            stream.send(StreamMessage::Delta(chunk.delta_content))?;
        }
        println!("Received content: {}", content);
        context.set("result", serde_json::Value::String(content.clone()));

        Ok(serde_json::json!({ "status": "editor" }))
    }
    async fn after_exec(
        &self,
        context: &mut Context,
        result: &Result<Value>,
    ) -> Result<NodeResult<Self::FlowStatus>> {
        println!("after_exec: {:?}", result);
        Ok(NodeResult {
            status: JobStatus::Finished,
            message: "done".to_owned(),
        })
    }
}
