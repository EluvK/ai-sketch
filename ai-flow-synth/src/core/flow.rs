use std::{collections::HashMap, sync::Arc};

use serde_json::Value;

use super::{
    context::{CONTEXT_RESULT, Context},
    node::Node,
    status::Status,
};

pub struct Flow<S: Status> {
    nodes: HashMap<String, Arc<dyn Node<FlowStatus = S>>>,
    edges: HashMap<String, Vec<(S, String)>>, // <from, [condition, to]>
    start_node: String,
}

impl<S: Status> Flow<S> {
    pub fn new(start_node_name: &str, start_node: Arc<dyn Node<FlowStatus = S>>) -> Self {
        let mut nodes = HashMap::new();
        nodes.insert(start_node_name.to_owned(), start_node);
        Flow {
            nodes,
            edges: HashMap::new(),
            start_node: start_node_name.to_owned(),
        }
    }

    pub fn add_node(&mut self, name: &str, node: Arc<dyn Node<FlowStatus = S>>) {
        self.nodes.insert(name.to_owned(), node);
    }

    pub fn add_edge(&mut self, from: &str, condition: S, to: &str) {
        self.edges
            .entry(from.to_owned())
            .or_default()
            .push((condition, to.to_owned()));
    }

    pub async fn run(&self, mut context: Context) -> anyhow::Result<Value> {
        let mut current_node_name = self.start_node.clone();
        while let Some(node) = self.nodes.get(&current_node_name) {
            // pre:
            node.prepare(&mut context).await?;
            // exec:
            let result = node.execute(&mut context).await;

            // after_exec:
            let result = node.after_exec(&mut context, &result).await?;

            if let Some(edges) = self.edges.get(&current_node_name) {
                // find the next node based on the result
                let mut found_next_node = false;
                for (condition, to) in edges {
                    if result.status == *condition {
                        current_node_name = to.clone();
                        found_next_node = true;
                        break;
                    }
                }
                if !found_next_node {
                    break; // no next node found, exit the loop
                }
            } else {
                break; // no edges for this node, exit the loop
            }
        }

        Ok(context.get(CONTEXT_RESULT).unwrap_or(&Value::Null).clone())
    }
}

#[macro_export]
macro_rules! flow {
    (start: ($name: expr, $start_node: expr)) => {{ $crate::core::flow::Flow::new($name, $start_node) }};
    (
        start: ($start_name: expr, $start_node: expr),
        nodes: [$(($name: expr, $node: expr)),* $(,)?],
    ) => {{
        let mut flow = $crate::core::flow::Flow::new($start_name, $start_node);
        $(
            flow.add_node($name, $node);
        )*
        flow
    }};
    (
        start: ($start_name: expr, $start_node: expr),
        nodes: [$(($name: expr, $node: expr)),* $(,)?],
        edges: [$(($from: expr, $condition: expr, $to: expr)),* $(,)?]
    ) => {{
        let mut flow = $crate::core::flow::Flow::new($start_name, $start_node);
        $(
            flow.add_node($name, $node);
        )*
        $(
            flow.add_edge($from, $condition, $to);
        )*
        flow
    }};
}

#[cfg(test)]
mod tests {
    #![allow(unused_variables)]
    #![allow(dead_code)]

    #[allow(unused_imports)]
    use super::*;
    use crate::core::context::Context;

    #[derive(Default, PartialEq, Eq)]
    enum MyStatus {
        #[default]
        Done,
        Repeat,
        Failed,
    }
    impl Status for MyStatus {
        fn failed() -> Self {
            MyStatus::Failed
        }
    }
    struct StartNode {}
    #[async_trait::async_trait]
    impl Node for StartNode {
        type FlowStatus = MyStatus;

        async fn execute(&self, context: &mut Context) -> anyhow::Result<Value> {
            Ok(Value::Null)
        }
    }
    struct EndNode {}
    #[async_trait::async_trait]
    impl Node for EndNode {
        type FlowStatus = MyStatus;

        async fn execute(&self, context: &mut Context) -> anyhow::Result<Value> {
            Ok(Value::Null)
        }
    }

    #[test]
    fn test_flow_macro() {
        let start_node = Arc::new(StartNode {});
        let end_node = Arc::new(EndNode {});
        let f1 = flow!(start:("start", start_node));

        let start_node = Arc::new(StartNode {});
        let end_node = Arc::new(EndNode {});
        let f2 = flow! {
            start: ("start", start_node),
            nodes: [("end", end_node)],
        };

        let start_node = Arc::new(StartNode {});
        let end_node = Arc::new(EndNode {});
        let f3 = flow! {
            start: ("start", start_node),
            nodes: [("end", end_node)],
            edges: [
                ("start", MyStatus::Done, "end"),
                ("start", MyStatus::Repeat, "start"),
            ]
        };
    }
}
