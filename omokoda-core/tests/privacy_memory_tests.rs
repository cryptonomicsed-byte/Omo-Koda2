#[cfg(test)]
mod privacy_memory_tests {
    use omokoda_core::interpreter::Steward;
    use omokoda_core::parser::parse;
    use std::path::PathBuf;

    #[tokio::test]
    async fn private_thought_leaks_no_text_to_public_session() {
        let mut steward = Steward::new();
        steward.set_mock_provider("mock thought".to_string());
        steward.dispatch(parse(r#"birth "luna""#).unwrap()[0].clone()).await.unwrap();
        
        let secret = "HIDDEN_TREASURE_123";
        // By default think is private
        steward.dispatch(parse(&format!(r#"think "{}""#, secret)).unwrap()[0].clone()).await.unwrap();
        
        let agent = steward.agent_state().unwrap();
        let public_json = serde_json::to_string(&agent.session()).unwrap();
        
        assert!(!public_json.contains(secret));
        assert!(!public_json.contains("mock thought"));
    }

    #[tokio::test]
    async fn private_thought_requires_unlocked_session_after_restart() {
        let mut steward = Steward::new();
        steward.set_mock_provider("mock thought".to_string());
        steward.dispatch(parse(r#"birth "luna""#).unwrap()[0].clone()).await.unwrap();
        let agent_id = steward.agent_state().unwrap().id().to_string();

        // Private think while unlocked (born unlocked)
        steward.dispatch(parse(r#"think "private message""#).unwrap()[0].clone()).await.unwrap();
        
        // Seal it for the first time with a password
        steward.dispatch(parse("/seal password").unwrap()[0].clone()).await.unwrap();

        // Restart steward and load agent
        let mut new_steward = Steward::new();
        new_steward.set_mock_provider("mock thought".to_string());
        new_steward.load_agent(&agent_id).unwrap();
        
        // Try to think privately without unlock
        let result = new_steward.dispatch(parse(r#"think "another private""#).unwrap()[0].clone()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("locked"));
        
        // Unlock and try again
        new_steward.dispatch(parse("/unlock password").unwrap()[0].clone()).await.unwrap();
        new_steward.dispatch(parse(r#"think "now it works""#).unwrap()[0].clone()).await.unwrap();
        
        // Cleanup
        let path = PathBuf::from("sessions").join(format!("{}.json", agent_id));
        let _ = std::fs::remove_file(path);
    }

    #[tokio::test]
    async fn sandbox_act_leaks_no_text_to_public_session() {
        let mut steward = Steward::new();
        steward.set_mock_provider("mock thought".to_string());
        steward.dispatch(parse(r#"birth "luna""#).unwrap()[0].clone()).await.unwrap();
        
        // Sandbox act
        let test_file = "test_sandbox_leak.txt";
        std::fs::write(test_file, "secret content").unwrap();
        
        steward.dispatch(parse(&format!(r#"act "read_file" "{}" /sandbox"#, test_file)).unwrap()[0].clone()).await.unwrap();
        
        let agent = steward.agent_state().unwrap();
        let public_json = serde_json::to_string(&agent.session()).unwrap();
        
        assert!(!public_json.contains("secret content"));
        
        std::fs::remove_file(test_file).unwrap();
    }

    #[tokio::test]
    async fn slash_seal_clears_private_data_from_memory() {
        let mut steward = Steward::new();
        steward.set_mock_provider("mock thought".to_string());
        steward.dispatch(parse(r#"birth "luna""#).unwrap()[0].clone()).await.unwrap();
        
        // Born unlocked, so we can think privately
        steward.dispatch(parse(r#"think "private""#).unwrap()[0].clone()).await.unwrap();
        
        assert!(steward.agent_state().unwrap().private_data().is_some());
        
        // Seal it
        steward.dispatch(parse("/seal secret").unwrap()[0].clone()).await.unwrap();
        
        assert!(steward.agent_state().unwrap().private_data().is_none());
    }
}
