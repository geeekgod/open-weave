use crate::error::{Result, WeaveError};
use crate::llm::{LLMProvider, Message, Role};
use crate::memory::short_term::ShortTermMemory;
use crate::memory::Memory;
use crate::planner::react::ReActPlanner;
use crate::planner::Planner;
use crate::tools::executor::ToolExecutor;
use crate::tools::registry::ToolRegistry;
use std::sync::Arc;
use uuid::Uuid;

pub struct AgentConfig {
    pub max_iterations: usize,
    pub system_prompt: String,
    pub timeout_secs: u64,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_iterations: 10,
            system_prompt: "You are a helpful AI assistant.".into(),
            timeout_secs: 60,
        }
    }
}

pub struct AgentOutput {
    pub content: String,
    pub iterations_used: usize,
    pub tool_calls_made: usize,
    pub duration_ms: u64,
}

pub struct Agent {
    pub id: Uuid,
    llm: Arc<dyn LLMProvider>,
    tools: Arc<ToolRegistry>,
    planner: Arc<dyn Planner>,
    config: AgentConfig,
}

impl Agent {
    pub fn new(llm: Arc<dyn LLMProvider>) -> Self {
        Self {
            id: Uuid::new_v4(),
            llm,
            tools: Arc::new(ToolRegistry::new()),
            planner: Arc::new(ReActPlanner::new()),
            config: AgentConfig::default(),
        }
    }

    pub fn with_config(mut self, config: AgentConfig) -> Self {
        self.config = config;
        self
    }

    pub fn register_tool(&mut self, tool: impl crate::tools::Tool + 'static) {
        if let Some(registry) = Arc::get_mut(&mut self.tools) {
            registry.register(tool);
        }
    }

    pub async fn run(&self, input: &str) -> Result<AgentOutput> {
        let mut memory = ShortTermMemory::default();
        memory.add(Message {
            role: Role::System,
            content: self.config.system_prompt.clone(),
            tool_calls: None,
        })?;
        memory.add(Message {
            role: Role::User,
            content: input.into(),
            tool_calls: None,
        })?;

        let executor = ToolExecutor::new(self.tools.clone());
        let schemas = self.tools.get_schemas();
        let mut tool_calls_made = 0;

        let start_time = std::time::Instant::now();

        for i in 0..self.config.max_iterations {
            let context = memory.get_context();
            
            // Ask planner for next step. Default ReAct uses LLM directly.
            let step = self.planner.plan(&context);

            match step {
                crate::planner::PlanStep::Respond(_text) => {
                    // Fallback or early termination from planner
                    // The standard ReAct uses the LLM complete.
                }
                _ => {}
            }

            let msg = self.llm.complete(&context, &schemas).await?;
            
            memory.add(msg.clone())?;

            if let Some(calls) = msg.tool_calls {
                if calls.is_empty() {
                    return Ok(AgentOutput {
                        content: msg.content,
                        iterations_used: i + 1,
                        tool_calls_made,
                        duration_ms: start_time.elapsed().as_millis() as u64,
                    });
                }

                tool_calls_made += calls.len();
                let results = executor.execute_all(calls.clone()).await;
                
                for (call, result) in calls.iter().zip(results.into_iter()) {
                    let content = result.unwrap_or_else(|e| format!("Error: {}", e));
                    memory.add(Message {
                        role: Role::Tool,
                        content,
                        tool_calls: Some(vec![call.clone()]),
                    })?;
                }
            } else {
                return Ok(AgentOutput {
                    content: msg.content,
                    iterations_used: i + 1,
                    tool_calls_made,
                    duration_ms: start_time.elapsed().as_millis() as u64,
                });
            }
        }

        Err(WeaveError::MaxIterationsReached)
    }
}