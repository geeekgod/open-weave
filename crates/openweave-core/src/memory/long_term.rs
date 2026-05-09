use super::Memory;
use crate::error::Result;
use crate::llm::Message;

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
    
    // Naive cosine similarity for MVP
    pub fn search(&self, _embedding: &[f32], _limit: usize) -> Vec<String> {
        // TODO: implement cosine similarity
        Vec::new()
    }
}

impl Memory for LongTermMemory {
    fn add(&mut self, _message: Message) -> Result<()> {
        // TODO: Need embedding function injection
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