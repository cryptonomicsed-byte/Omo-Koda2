#[cfg(test)]
mod rack_tests {
    use omokoda_core::identity::AgentId;
    use omokoda_core::session::{ContentBlock, ConversationMessage, MessageRole, Session};

    #[test]
    fn public_messages_compression_level_1() {
        let mut session = Session::new(AgentId::new("0123456789abcdefghij"), "luna".to_string(), 0);

        // Large tool result to trigger ContentReplace (Level 1)
        let large_output = "A".repeat(5000);
        let msg = ConversationMessage {
            role: MessageRole::Tool,
            blocks: vec![ContentBlock::ToolResult {
                tool_use_id: "test".to_string(),
                output: large_output,
                is_error: false,
            }],
            is_private: false,
            timestamp: 0,
        };

        session.push_public(msg, 0.0);

        if let ContentBlock::ToolResult { output, .. } = &session.public_messages[0].blocks[0] {
            assert!(output.len() <= 2025); // 2000 + truncation message
            assert!(output.contains("[TRUNCATED]"));
        } else {
            panic!("Unexpected content block type");
        }
    }

    #[test]
    fn public_messages_compression_level_2() {
        let mut session = Session::new(AgentId::new("0123456789abcdefghij"), "luna".to_string(), 0);

        // Push many non-essential messages to exceed Level 2 threshold (8000 chars)
        for _ in 0..100 {
            session.push_public(ConversationMessage::user_text(&"B".repeat(100)), 0.0);
        }

        // Snip should have removed some messages
        assert!(session.public_messages.len() < 100);
    }
}
