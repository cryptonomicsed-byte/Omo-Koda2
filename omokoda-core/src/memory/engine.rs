use crate::interpreter::MemoryEntry;

pub struct MemoryEngine;

impl MemoryEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn process_working_memory(&self, memory: &mut Vec<MemoryEntry>) {
        // Prune short-term memory (low importance)
        if memory.len() > 100 {
            memory.sort_by(|a, b| b.importance.partial_cmp(&a.importance).unwrap());
            memory.truncate(100);
        }
    }

    pub fn compress(
        &self,
        messages: &mut Vec<crate::session::ConversationMessage>,
        _reputation: f64,
    ) {
        // Level 1: Truncate messages exceeding 2000 chars
        for message in messages.iter_mut() {
            for block in &mut message.blocks {
                match block {
                    crate::session::ContentBlock::ToolResult { output, .. } => {
                        if output.len() > 2000 {
                            *output = format!("{}... [TRUNCATED]", &output[..2000]);
                        }
                    }
                    crate::session::ContentBlock::Text { text } => {
                        if text.len() > 5000 {
                            *text = format!("{}... [TRUNCATED]", &text[..5000]);
                        }
                    }
                    _ => {}
                }
            }
        }

        // Level 2: Keep last 50 messages
        if messages.len() > 50 {
            messages.drain(0..(messages.len() - 50));
        }
    }
}
