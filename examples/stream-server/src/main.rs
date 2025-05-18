mod node;

use std::sync::Arc;

use ai_flow_synth::core::stream_message::StreamMessage;
use ai_flow_synth::{core::context::Context, flow};

use futures_util::StreamExt;
use node::{EditorNode, JobStatus, WriterNode};

use serde::{Deserialize, Serialize};

use salvo::prelude::*;
use salvo::sse::{SseEvent, SseKeepAlive};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let router = Router::new()
        .goal(index)
        .push(Router::with_path("test").post(llm_chat));

    let acceptor = TcpListener::new("0.0.0.0:5800").bind().await;
    Server::new(acceptor).serve(router).await;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatRequest {
    pub prompt: String,
    // pub model: String,
    // pub temperature: f32,
    // pub top_p: f32,
    // pub max_tokens: usize,
}

#[handler]
async fn llm_chat(req: &mut Request, res: &mut Response) {
    let question = req.parse_body::<ChatRequest>().await.unwrap();
    tracing::info!("llm_chat: {:?}", question);
    let editor_prompt = "Edit the story to make it more emotional and engaging.";
    let writer_node = Arc::new(WriterNode::new(question.prompt.to_string()));
    let editor_node = Arc::new(EditorNode::new(editor_prompt.to_string()));
    let context = Context::new();

    let flow = flow!(
        start: ("start", writer_node),
        nodes: [("editor", editor_node)],
        edges: [("start", JobStatus::Written, "editor")]
    );

    let listener = context.listen();
    let stream = listener.map(|msg| match msg {
        Ok(msg) => {
            tracing::info!("Received message: {:?}", msg);
            match msg {
                StreamMessage::Delta(delta) => {
                    // tracing::info!("Delta: {:?}", delta);
                    Ok::<_, salvo::Error>(SseEvent::default().text(delta))
                } // _ => Ok::<_, salvo::Error>(SseEvent::default().text("")),
            }
        }
        Err(e) => {
            tracing::error!("Error receiving message: {:?}", e);
            Err(salvo::Error::Other(Box::new(e)))
        }
    });
    SseKeepAlive::new(stream).stream(res);
    tokio::spawn(async move {
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
    });
}

#[handler]
async fn index(res: &mut Response) {
    res.render(Text::Html(INDEX_HTML));
}

static INDEX_HTML: &str = include_str!("index.html");
