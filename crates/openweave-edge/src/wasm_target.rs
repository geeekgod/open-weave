#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub async fn run_agent(input: &str) -> String {
    // WASM specific agent invocation here
    format!("WASM agent received: {}", input)
}