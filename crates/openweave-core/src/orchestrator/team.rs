use crate::agent::{Agent, AgentOutput};
use crate::error::{Result, WeaveError};
use std::collections::HashMap;

pub struct AgentTeam {
    agents: HashMap<String, (Agent, String)>, // name -> (agent, description)
}

impl Default for AgentTeam {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentTeam {
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
        }
    }

    pub fn register_agent(&mut self, name: impl Into<String>, agent: Agent, description: impl Into<String>) {
        self.agents.insert(name.into(), (agent, description.into()));
    }

    pub fn route(&self, _task: &str) -> Result<&Agent> {
        // LLM routing logic here. Mocking return first agent.
        self.agents.values().next().map(|(a, _)| a).ok_or_else(|| WeaveError::LlmError("No agents".into()))
    }

    pub async fn run(&self, task: &str) -> Result<AgentOutput> {
        let agent = self.route(task)?;
        agent.run(task).await
    }
}