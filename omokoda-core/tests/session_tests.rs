#[cfg(test)]
mod session_tests {
    use omokoda_core::session::{ContentBlock, ConversationMessage, Session, SessionError};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn session_starts_with_current_version() {
        let session = Session::new();
        assert_eq!(session.version, 1);
        assert!(session.messages.is_empty());
    }

    #[test]
    fn session_save_and_load_roundtrip() {
        let path = temp_session_path("roundtrip");
        let mut session = Session::new();
        session.push_message(ConversationMessage::user_text("birth luna"));
        session.push_message(ConversationMessage::assistant_text("born"));
        session.push_message(ConversationMessage {
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
    fn session_rejects_unknown_version() {
        let path = temp_session_path("bad-version");
        fs::write(&path, r#"{"version":999,"messages":[]}"#).unwrap();

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
