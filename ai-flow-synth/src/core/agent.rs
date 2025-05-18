use serde::{Deserialize, Serialize};

// The agent should control the flow of the task,
// so it should contains all the nodes and the flow of the task
// also the input and output information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub description: String,
}
