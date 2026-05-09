pub mod react;

use crate::llm::{Message, ToolCall};
use uuid::Uuid;

pub enum PlanStep {
    UseTool(ToolCall),
    Respond(String),
    Delegate(Uuid),
}

pub trait Planner: Send + Sync {
    fn plan(&self, context: &[Message]) -> PlanStep;
}