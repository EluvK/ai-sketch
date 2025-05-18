use serde_json::Value;

use super::{
    context::{CONTEXT_RESULT, Context},
    status::Status,
};

#[async_trait::async_trait]
pub trait Node: Send + Sync {
    type FlowStatus: Status;

    #[allow(unused_variables)]
    async fn prepare(&self, context: &mut Context) -> anyhow::Result<()> {
        Ok(())
    }

    async fn execute(&self, context: &mut Context) -> anyhow::Result<Value>;

    #[allow(unused_variables)]
    async fn after_exec(
        &self,
        context: &mut Context,
        result: &anyhow::Result<Value>,
    ) -> anyhow::Result<NodeResult<Self::FlowStatus>> {
        match result {
            Ok(value) => {
                context.set(CONTEXT_RESULT, value.clone());
                Ok(NodeResult::default())
            }
            Err(e) => Ok(NodeResult {
                status: Self::FlowStatus::failed(),
                message: e.to_string(),
            }),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct NodeResult<S: Status> {
    pub status: S,
    pub message: String,
}
