use super::Memory;
use crate::error::Result;
use crate::llm::Message;
use std::collections::VecDeque;

pub struct ShortTermMemory {
    messages: VecDeque<Message>,
    max_messages: usize,
}

impl ShortTermMemory {
    pub fn new(max_messages: usize) -> Self {
        Self {
            messages: VecDeque::with_capacity(max_messages),
            max_messages,
        }
    }
}

impl Default for ShortTermMemory {
    fn default() -> Self {
        Self::new(20)
    }
}

impl Memory for ShortTermMemory {
    fn add(&mut self, message: Message) -> Result<()> {
        if self.messages.len() >= self.max_messages {
            self.messages.pop_front();
        }
        self.messages.push_back(message);
        Ok(())
    }

    fn get_context(&self) -> Vec<Message> {
        self.messages.iter().cloned().collect()
    }

    fn clear(&mut self) {
        self.messages.clear();
    }
}