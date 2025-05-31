use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct TaskHistory {
    pub evaluation_previous_goal: String,
    pub memory: String,
    pub next_goal: String,
    pub action: Value,
    pub done: bool,
    pub final_result: Option<String>,
}
