use openweave_core::agent::{Agent, AgentConfig};
use openweave_core::orchestrator::team::AgentTeam;
use openweave_core::llm::openai::OpenAIProvider;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut team = AgentTeam::new();
    let llm = Arc::new(OpenAIProvider::new("gpt-4o"));

    let coder = Agent::new(llm.clone()).with_config(AgentConfig {
        system_prompt: "You are a coder.".into(),
        ..Default::default()
    });
    
    let reviewer = Agent::new(llm).with_config(AgentConfig {
        system_prompt: "You are a reviewer.".into(),
        ..Default::default()
    });

    team.register_agent("coder", coder, "Writes code");
    team.register_agent("reviewer", reviewer, "Reviews code");

    let res = team.run("Write a python script for scraping").await?;
    println!("Team result: {}", res.content);
    
    Ok(())
}