use std::sync::Arc;

use ai_flow_synth::{core::context::Context, flow};

mod node;

use node::{EditorNode, JobStatus, WriterNode};
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() {
    let write_prompt = "Write a short story about a robot learning to love, in 100 words.";
    let editor_prompt = "Edit the story to make it more emotional and engaging.";
    let writer_node = Arc::new(WriterNode::new(write_prompt.to_string()));
    let editor_node = Arc::new(EditorNode::new(editor_prompt.to_string()));
    let context = Context::new();

    let flow = flow!(
        start: ("start", writer_node),
        nodes: [("editor", editor_node)],
        edges: [("start", JobStatus::Written, "editor")]
    );

    let mut listener = context.listen();
    tokio::spawn(async move {
        while let Some(message) = listener.next().await {
            match message {
                Ok(msg) => println!("Received message: {:?}", msg),
                Err(e) => eprintln!("Error receiving message: {:?}", e),
            }
        }
    });

    let result = flow.run(context.clone()).await;
    match result {
        Ok(value) => println!("Flow executed successfully: {:?}", value),
        Err(e) => eprintln!("Flow execution failed: {:?}", e),
    }

    let result = context.get("result");
    match result {
        Some(value) => println!("Result: {:?}", value),
        None => eprintln!("No result found"),
    }
}
