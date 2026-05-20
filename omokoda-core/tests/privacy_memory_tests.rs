#[cfg(test)]
mod privacy_memory_tests {
    use omokoda_core::interpreter::Steward;
    use omokoda_core::parser::parse;

    #[tokio::test]
    async fn private_thought_leaks_no_text_to_public_session() {
        let mut steward = Steward::new();
        steward.set_mock_provider("mock thought".to_string());
        steward
            .dispatch(parse(r#"birth "luna""#).unwrap()[0].clone())
            .await
            .unwrap();

        let secret = "HIDDEN_TREASURE_123";
        // By default think is private
        steward
            .dispatch(parse(&format!(r#"think "{}""#, secret)).unwrap()[0].clone())
            .await
            .unwrap();

        let agent = steward.agent_core().unwrap();
        let public_json = serde_json::to_string(&agent.session()).unwrap();

        assert!(!public_json.contains(secret));
        assert!(!public_json.contains("mock thought"));
    }

    #[tokio::test]
    async fn private_thought_requires_unlocked_session_after_restart() {
        let mut steward = Steward::new();
        let session_dir = std::env::current_dir()
            .unwrap()
            .join("test_sessions_privacy_restart");
        steward.set_session_dir(session_dir.clone());
        steward.set_mock_provider("mock thought".to_string());
        steward
            .dispatch(parse(r#"birth "luna""#).unwrap()[0].clone())
            .await
            .unwrap();
        let agent_id = steward.agent_core().unwrap().id().clone();

        // Private think while unlocked (born unlocked)
        steward
            .dispatch(parse(r#"think "private message""#).unwrap()[0].clone())
            .await
            .unwrap();

        // Seal it for the first time with a password
        steward
            .dispatch(parse("/seal password").unwrap()[0].clone())
            .await
            .unwrap();

        // Restart steward and load agent
        let mut new_steward = Steward::new();
        new_steward.set_session_dir(session_dir.clone());
        new_steward.set_mock_provider("mock thought".to_string());
        new_steward.load_agent(&agent_id).unwrap();

        // Try to think privately without unlock
        let result = new_steward
            .dispatch(parse(r#"think "another private""#).unwrap()[0].clone())
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("locked"));

        // Unlock and try again
        new_steward
            .dispatch(parse("/unlock password").unwrap()[0].clone())
            .await
            .unwrap();
        new_steward
            .dispatch(parse(r#"think "now it works""#).unwrap()[0].clone())
            .await
            .unwrap();

        // Cleanup
        let _ = std::fs::remove_dir_all(session_dir);
    }

    #[tokio::test]
    async fn sandbox_act_leaks_no_text_to_public_session() {
        let mut steward = Steward::new();
        steward.set_mock_provider("mock thought".to_string());
        steward
            .dispatch(parse(r#"birth "luna""#).unwrap()[0].clone())
            .await
            .unwrap();

        // Sandbox act
        let test_file = "test_sandbox_leak.txt";
        std::fs::write(test_file, "secret content").unwrap();

        steward
            .dispatch(
                parse(&format!(r#"act "read_file" "{}" /sandbox"#, test_file)).unwrap()[0].clone(),
            )
            .await
            .unwrap();

        let agent = steward.agent_core().unwrap();
        let public_json = serde_json::to_string(&agent.session()).unwrap();

        assert!(!public_json.contains("secret content"));

        std::fs::remove_file(test_file).unwrap();
    }

    #[tokio::test]
    async fn slash_seal_clears_private_data_from_memory() {
        let mut steward = Steward::new();
        steward.set_mock_provider("mock thought".to_string());
        steward
            .dispatch(parse(r#"birth "luna""#).unwrap()[0].clone())
            .await
            .unwrap();

        // Born unlocked, so we can think privately
        steward
            .dispatch(parse(r#"think "private""#).unwrap()[0].clone())
            .await
            .unwrap();

        assert!(steward.agent_core().unwrap().private_data().is_some());

        // Seal it
        steward
            .dispatch(parse("/seal secret").unwrap()[0].clone())
            .await
            .unwrap();

        assert!(steward.agent_core().unwrap().private_data().is_none());
    }

    #[tokio::test]
    async fn seal_and_unlock_emit_audit_events() {
        use omokoda_core::interpreter::TurnEvent;

        let mut steward = Steward::new();
        steward.set_mock_provider("mock thought".to_string());
        steward
            .dispatch(parse(r#"birth "luna""#).unwrap()[0].clone())
            .await
            .unwrap();
        steward
            .dispatch(parse(r#"think "private""#).unwrap()[0].clone())
            .await
            .unwrap();

        let (tx, mut rx) = tokio::sync::mpsc::channel(8);
        steward
            .dispatch_with_event_sink(parse("/seal secret").unwrap()[0].clone(), tx)
            .await
            .unwrap();
        assert!(matches!(rx.recv().await, Some(TurnEvent::Started)));
        assert!(
            matches!(rx.recv().await, Some(TurnEvent::Audit(event)) if event == "private_session_sealed")
        );

        let (tx, mut rx) = tokio::sync::mpsc::channel(8);
        steward
            .dispatch_with_event_sink(parse("/unlock secret").unwrap()[0].clone(), tx)
            .await
            .unwrap();
        assert!(matches!(rx.recv().await, Some(TurnEvent::Started)));
        assert!(
            matches!(rx.recv().await, Some(TurnEvent::Audit(event)) if event == "private_session_unsealed")
        );
    }
}
