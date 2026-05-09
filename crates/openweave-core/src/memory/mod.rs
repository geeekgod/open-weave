pub mod long_term;
pub mod short_term;

use crate::error::Result;
use crate::llm::Message;

pub trait Memory: Send + Sync {
    fn add(&mut self, message: Message) -> Result<()>;
    fn get_context(&self) -> Vec<Message>;
    fn clear(&mut self);
}