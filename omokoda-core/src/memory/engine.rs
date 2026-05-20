pub struct MemoryEngine;

impl MemoryEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn compress(
        &self,
        messages: &mut Vec<crate::session::ConversationMessage>,
        _reputation: f64,
    ) {
        // Level 1: Truncate messages exceeding 1000 chars
        for message in messages.iter_mut() {
            for block in &mut message.blocks {
                if let crate::session::ContentBlock::ToolResult { output, .. } = block {
                    if output.len() > 1000 {
                        *output = format!("{}... [TRUNCATED]", &output[..1000]);
                    }
                }
            }
        }

        // Level 2: If too many messages, drop oldest
        if messages.len() > 80 {
            messages.drain(0..(messages.len() - 80));
        }
    }
}
