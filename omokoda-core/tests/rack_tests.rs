#[cfg(test)]
mod rack_tests {
    use omokoda_core::session::{Session, ConversationMessage, PrivateSessionData, ContentBlock, MessageRole};
    use omokoda_core::identity::AgentId;
    use omokoda_core::identity::odu::{OduSeed, OduIdentity};

    #[test]
    fn public_messages_evicts_after_100() {
        let mut session = Session::new(AgentId::new("0123456789abcdefghij"), "luna".to_string(), 0);
        
        for i in 0..100 {
            session.push_public(ConversationMessage::user_text(&format!("msg {}", i)));
        }
        assert_eq!(session.public_messages.len(), 100);
        
        // 101st message should trigger eviction
        session.push_public(ConversationMessage::user_text("msg 101"));
        
        // Should have 100 + 1 - 20 = 81 messages
        assert_eq!(session.public_messages.len(), 81);
        
        // The first remaining message should be "msg 20"
        if let ContentBlock::Text { text } = &session.public_messages[0].blocks[0] {
            assert_eq!(text, "msg 20");
        } else {
            panic!("Unexpected content block type");
        }
    }

    #[test]
    fn private_messages_evicts_and_summarizes_after_1000() {
        let mut private_data = PrivateSessionData {
            odu_seed: OduSeed::new([0u8; 32]),
            odu_identity: OduIdentity { primary_index: 0, mnemonic: "test".to_string() },
            private_messages: Vec::new(),
        };

        for i in 0..1000 {
            private_data.push_private(ConversationMessage::user_text(&format!("msg {}", i)));
        }
        assert_eq!(private_data.private_messages.len(), 1000);

        // 1001st message should trigger eviction and summary
        private_data.push_private(ConversationMessage::user_text("msg 1001"));

        assert_eq!(private_data.private_messages.len(), 802);

        // The first message should be the summary
        assert_eq!(private_data.private_messages[0].role, MessageRole::System);
        if let ContentBlock::Text { text } = &private_data.private_messages[0].blocks[0] {
            assert!(text.contains("SUMMARY"));
        } else {
            panic!("Unexpected content block type");
        }

        // The second message should be "msg 200"
        if let ContentBlock::Text { text } = &private_data.private_messages[1].blocks[0] {
            assert_eq!(text, "msg 200");
        } else {
            panic!("Unexpected content block type");
        }
    }
}
