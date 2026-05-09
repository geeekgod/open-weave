use openweave_core::agent::{Agent, AgentConfig};
use openweave_core::llm::openai::OpenAIProvider;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let llm = Arc::new(OpenAIProvider::new("gpt-4o"));
    let agent = Agent::new(llm).with_config(AgentConfig {
        system_prompt: "You are a research agent.".into(),
        ..Default::default()
    });
    
    // In a full implementation, register tools here:
    // agent.register_tool(WebSearchTool::new());
    
    let res = agent.run("Research the history of Rust programming language").await?;
    println!("Result: {}", res.content);
    println!("Used {} iterations, {} ms", res.iterations_used, res.duration_ms);
    
    Ok(())
}