use openweave_core::agent::{Agent, AgentConfig};
use openweave_core::llm::groq::GroqProvider;
use std::sync::Arc;

#[path = "../../tools/web_search.rs"]
mod web_search;

#[path = "../../tools/file_ops.rs"]
mod file_ops;

/// Runs a sample research agent that queries the web about Rust's history using a Groq LLM
/// and writes the answer to `research.txt`.
///
/// This function:
/// - Loads `GROQ_API_KEY` from a local `.env` file (preferring a line starting with `GROQ_API_KEY=`),
///   falling back to the `GROQ_API_KEY` environment variable.
/// - Creates a `GroqProvider` LLM and an `Agent` configured to use `web_search` and `file_ops` tools,
///   with a maximum of 5 iterations.
/// - Registers the `web_search` and `file_ops` tools, runs the agent on a research prompt,
///   prints the final result and metrics, then prints and removes `research.txt` if created.
///
/// # Examples
///
/// ```
/// // Run the async `main` from a synchronous test/example harness.
/// let rt = tokio::runtime::Runtime::new().unwrap();
/// rt.block_on(async {
///     // `main` returns a Result; propagate failure in examples.
///     crate::main().await.unwrap();
/// });
/// ```
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let key = std::fs::read_to_string(".env")
        .unwrap_or_default()
        .lines()
        .find(|line| line.starts_with("GROQ_API_KEY="))
        .map(|line| line.trim_start_matches("GROQ_API_KEY=").trim().to_string())
        .unwrap_or_else(|| std::env::var("GROQ_API_KEY").unwrap_or_default());

    let llm = Arc::new(GroqProvider::new("openai/gpt-oss-20b").with_api_key(key));
    
    let mut agent = Agent::new(llm).with_config(AgentConfig {
        system_prompt: "You are a concise research agent with access to tools. When asked to research, use the web_search tool. Then use the file_ops tool to write the result to research.txt.".into(),
        max_iterations: 5,
        ..Default::default()
    });
    
    // Register tools
    agent.register_tool(web_search::WebSearchTool::new());
    agent.register_tool(file_ops::FileOpsTool::new());
    
    println!("Running research agent...");
    let res = agent.run("Research the history of the Rust programming language and write the answer to 'research.txt'").await?;
    println!("Final Result: {}", res.content);
    println!("Used {} iterations, {} ms", res.iterations_used, res.duration_ms);
    
    if std::path::Path::new("research.txt").exists() {
        println!("Content of research.txt:\n{}", std::fs::read_to_string("research.txt").unwrap());
        let _ = std::fs::remove_file("research.txt");
    }
    
    Ok(())
}