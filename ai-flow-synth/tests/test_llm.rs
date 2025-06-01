use ai_flow_synth::{
    core::{context::Context, stream_message::StreamMessage},
    llm::{chat, model::ChatMessage, provider::deepseek::DeepSeekClient, tool::ToolRegistry},
    utils::{LogConfig, enable_log},
};
use serde::{Deserialize, Serialize};
use tokio_stream::StreamExt;

#[derive(Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetWeatherParams {
    pub location: String,
}

fn get_weather(params: GetWeatherParams) -> serde_json::Value {
    serde_json::Value::String(format!(
        "The weather in {} is sunny. The current time is 2025.05.20 10:00am",
        params.location
    ))
}

#[tokio::test]
async fn test_llm_function_call() -> anyhow::Result<()> {
    let log_config = LogConfig::default();
    let _g = enable_log(&log_config).unwrap();
    let mut client = DeepSeekClient::default();

    let mut registry = ToolRegistry::new();
    registry.register::<GetWeatherParams, _>("get_weather", "获取天气", get_weather);

    client.add_tools(registry.export_all_tools());
    let mut context = Context::new();
    let mut listener = context.listen();
    tokio::spawn(async move {
        while let Some(msg) = listener.next().await {
            match msg {
                Ok(StreamMessage::Delta(delta)) => {
                    println!("Received delta: {}", delta);
                }
                Ok(StreamMessage::Procedure(proc)) => {
                    println!("Received procedure: {}", proc);
                }
                Err(e) => eprintln!("Error receiving message: {}", e),
            }
        }
    });

    let messages = vec![
        ChatMessage::system("你是一个辅助助手，能够调取相应的工具来回答用户的问题。"),
        ChatMessage::user(
            "北京的天气怎么样？",
            // "Hi, would you please tell me what the time is it now, and weather in HangZhou",
        ),
    ];
    let final_result = chat(messages, context.stream("stream_id"), &client, &registry).await?;

    println!("Final result: {}", final_result);
    Ok(())
}
