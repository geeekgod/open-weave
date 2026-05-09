use openweave_core::agent::{Agent, AgentConfig};
use openweave_core::llm::groq::GroqProvider;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read the .env file directly manually for the example
    let key = std::fs::read_to_string(".env")
        .unwrap_or_default()
        .lines()
        .find(|line| line.starts_with("GROQ_API_KEY="))
        .map(|line| line.trim_start_matches("GROQ_API_KEY=").trim().to_string())
        .unwrap_or_else(|| std::env::var("GROQ_API_KEY").unwrap_or_default());

    let llm = Arc::new(GroqProvider::new("openai/gpt-oss-20b").with_api_key(key));
    
    let agent = Agent::new(llm).with_config(AgentConfig {
        system_prompt: "You are a concise research agent.".into(),
        ..Default::default()
    });
    
    println!("Running research agent...");
    let res = agent.run("In 2 sentences, what is the Rust programming language?").await?;
    println!("Result: {}", res.content);
    println!("Used {} iterations, {} ms", res.iterations_used, res.duration_ms);
    
    Ok(())
}