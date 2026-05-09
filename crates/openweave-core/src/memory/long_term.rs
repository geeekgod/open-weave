use super::Memory;
use crate::error::Result;
use crate::llm::Message;
use std::cmp::Ordering;

pub struct LongTermMemory {
    // text, embedding, metadata
    records: Vec<(String, Vec<f32>, String)>,
}

impl LongTermMemory {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }
    
    pub fn search(&self, query_embedding: &[f32], limit: usize) -> Vec<String> {
        let mut scored_records: Vec<(&String, f32)> = self.records
            .iter()
            .map(|(text, emb, _meta)| {
                let score = cosine_similarity(query_embedding, emb);
                (text, score)
            })
            .collect();

        // Sort descending by score
        scored_records.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));

        scored_records
            .into_iter()
            .take(limit)
            .map(|(text, _score)| text.clone())
            .collect()
    }

    // Direct insertion method bypassing standard add() since we don't have embedder built-in
    pub fn insert(&mut self, text: String, embedding: Vec<f32>, metadata: String) {
        self.records.push((text, embedding, metadata));
    }
}

impl Default for LongTermMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl Memory for LongTermMemory {
    fn add(&mut self, _message: Message) -> Result<()> {
        // Core trait assumes standard messages. To do this properly requires 
        // an async embedder injected into the memory, or passing embeddings.
        Ok(())
    }

    fn get_context(&self) -> Vec<Message> {
        // Context retrieval should be query-based for LTM
        Vec::new()
    }

    fn clear(&mut self) {
        self.records.clear();
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let mut dot_product = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;

    for (x, y) in a.iter().zip(b.iter()) {
        dot_product += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot_product / (norm_a.sqrt() * norm_b.sqrt())
}