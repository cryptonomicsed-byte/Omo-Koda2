#[cfg(test)]
mod session_tests {
    use omokoda_core::interpreter::AgentState;
    use omokoda_core::session::{
        ContentBlock, ConversationMessage, PrivateSessionData, Session, SessionError,
    };
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn session_starts_with_current_version() {
        let agent = AgentState::birth("luna".to_string());
        let session = Session::new(&agent);
        assert_eq!(session.version, 1);
        assert_eq!(session.name, "luna");
        assert!(session.public_messages.is_empty());
    }

    #[test]
    fn session_save_and_load_roundtrip() {
        let path = temp_session_path("roundtrip");
        let agent = AgentState::birth("luna".to_string());
        let mut session = Session::new(&agent);
        session.push_public(ConversationMessage::user_text("birth luna"));
        session.push_public(ConversationMessage::assistant_text("born"));
        session.push_public(ConversationMessage {
            role: omokoda_core::session::MessageRole::Assistant,
            blocks: vec![ContentBlock::ToolUse {
                id: "tool-1".to_string(),
                name: "web_search".to_string(),
                input: "bitcoin".to_string(),
            }],
        });

        session.save_to_path(&path).unwrap();
        let loaded = Session::load_from_path(&path).unwrap();
        fs::remove_file(&path).unwrap();

        assert_eq!(loaded, session);
    }

    #[test]
    fn session_encryption_roundtrip() {
        let agent = AgentState::birth("luna".to_string());
        let mut session = Session::new(&agent);
        let private_data = PrivateSessionData {
            odu_seed: vec![1, 2, 3, 4],
            private_messages: vec![ConversationMessage::user_text("secret thought")],
        };

        let passphrase = "correct horse battery staple";
        session.encrypt_private(&private_data, passphrase).unwrap();

        assert!(session.encrypted_private.is_some());

        let decrypted = session.decrypt_private(passphrase).unwrap();
        assert_eq!(decrypted, private_data);
    }

    #[test]
    fn session_decryption_fails_with_wrong_passphrase() {
        let agent = AgentState::birth("luna".to_string());
        let mut session = Session::new(&agent);
        let private_data = PrivateSessionData {
            odu_seed: vec![1, 2, 3, 4],
            private_messages: vec![],
        };

        session
            .encrypt_private(&private_data, "correct")
            .unwrap();
        let result = session.decrypt_private("wrong");

        assert!(matches!(result, Err(SessionError::Crypto)));
    }

    #[test]
    fn session_leakage_prevention() {
        let agent = AgentState::birth("luna".to_string());
        let mut session = Session::new(&agent);
        let secret_text = "HIDDEN_TREASURE_123";
        let private_data = PrivateSessionData {
            odu_seed: vec![0xDE, 0xAD, 0xBE, 0xEF],
            private_messages: vec![ConversationMessage::user_text(secret_text)],
        };

        session
            .encrypt_private(&private_data, "passphrase")
            .unwrap();
        let json = serde_json::to_string(&session).unwrap();

        assert!(!json.contains(secret_text));
        assert!(!json.contains("deadbeef"));
        assert!(!json.contains("odu_seed"));
        assert!(json.contains("encrypted_private"));
    }

    #[test]
    fn session_rejects_unknown_version() {
        let path = temp_session_path("bad-version");
        fs::write(
            &path,
            r#"{"version":999,"agent_id":"agent-1","name":"luna","birth_timestamp":0,"reputation":0.0,"public_messages":[]}"#,
        )
        .unwrap();

        let error = Session::load_from_path(&path).unwrap_err();
        fs::remove_file(&path).unwrap();

        assert!(matches!(error, SessionError::UnsupportedVersion(999)));
    }

    fn temp_session_path(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("omokoda-session-{label}-{nanos}.json"))
    }
}
