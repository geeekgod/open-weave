use super::{PlanStep, Planner};
use crate::llm::{Message, Role};

pub struct ReActPlanner;

impl Default for ReActPlanner {
    fn default() -> Self {
        Self::new()
    }
}

impl ReActPlanner {
    pub fn new() -> Self {
        Self
    }
}

impl Planner for ReActPlanner {
    fn plan(&self, context: &[Message]) -> PlanStep {
        // Find last assistant message to determine next step
        for msg in context.iter().rev() {
            if matches!(msg.role, Role::Assistant) {
                if let Some(calls) = &msg.tool_calls {
                    if !calls.is_empty() {
                        // Assuming first call for simple ReAct
                        return PlanStep::UseTool(calls[0].clone());
                    }
                }
                return PlanStep::Respond(msg.content.clone());
            }
        }
        PlanStep::Respond("No action determined".into())
    }
}