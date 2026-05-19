#[cfg(test)]
mod persistence_tests {
    use omokoda_core::interpreter::Steward;
    use omokoda_core::parser::parse;
    use std::path::PathBuf;

    fn test_session_dir(test_name: &str) -> PathBuf {
        let mut path = std::env::current_dir().unwrap();
        path.push("target");
        path.push("test_sessions");
        path.push(test_name);
        if path.exists() {
            let _ = std::fs::remove_dir_all(&path);
        }
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    #[tokio::test]
    async fn auto_save_creates_file_on_birth() {
        let session_dir = test_session_dir("auto_save_creates_file_on_birth");
        let mut steward = Steward::new().with_session_dir(session_dir);
        steward
            .dispatch(parse(r#"birth "luna""#).unwrap()[0].clone())
            .await
            .unwrap();

        let agent_id = steward.agent_state().unwrap().id();
        let path = steward.agent_storage_path(agent_id);

        assert!(path.exists());

        // Cleanup
        let _ = std::fs::remove_dir_all(steward.agent_storage_path(agent_id).parent().unwrap());
    }

    #[tokio::test]
    async fn persistence_roundtrip() {
        let session_dir = test_session_dir("persistence_roundtrip");
        let mut steward = Steward::new().with_session_dir(session_dir.clone());
        steward
            .dispatch(parse(r#"birth "luna""#).unwrap()[0].clone())
            .await
            .unwrap();

        let agent_id = steward.agent_state().unwrap().id().clone();
        let dna = steward.agent_state().unwrap().dna_fingerprint().to_string();

        // Start a new steward and load the agent
        let mut new_steward = Steward::new().with_session_dir(session_dir);
        new_steward.load_agent(&agent_id).unwrap();

        assert_eq!(new_steward.agent_state().unwrap().name(), "luna");
        assert_eq!(new_steward.agent_state().unwrap().dna_fingerprint(), dna);

        // Cleanup
        let path = new_steward.agent_storage_path(&agent_id);
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[tokio::test]
    async fn auto_save_updates_reputation() {
        let session_dir = test_session_dir("auto_save_updates_reputation");
        let mut steward = Steward::new().with_session_dir(session_dir.clone());
        steward
            .dispatch(parse(r#"birth "luna""#).unwrap()[0].clone())
            .await
            .unwrap();

        let agent_id = steward.agent_state().unwrap().id().clone();

        // Manually update reputation and check if it's saved
        // Tier 1 starts at 20.0
        steward.set_reputation_for_test(21.0);

        let mut new_steward = Steward::new().with_session_dir(session_dir);
        new_steward.load_agent(&agent_id).unwrap();
        assert_eq!(new_steward.reputation(), 21.0);

        // Cleanup
        let path = new_steward.agent_storage_path(&agent_id);
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[tokio::test]
    async fn load_agent_can_unlock_sealed_private_session() {
        let session_dir = test_session_dir("load_agent_can_unlock_sealed_private_session");
        let mut steward = Steward::new().with_session_dir(session_dir.clone());
        steward.set_mock_provider("mock thought".to_string());
        steward
            .dispatch(parse(r#"birth "luna""#).unwrap()[0].clone())
            .await
            .unwrap();
        steward
            .dispatch(parse(r#"think "secret thought""#).unwrap()[0].clone())
            .await
            .unwrap();
        steward
            .dispatch(parse(r#"/seal mypass"#).unwrap()[0].clone())
            .await
            .unwrap();

        let agent_id = steward.agent_state().unwrap().id().clone();
        let path = steward.agent_storage_path(&agent_id);
        assert!(path.exists());

        let mut new_steward = Steward::new().with_session_dir(session_dir);
        new_steward.load_agent(&agent_id).unwrap();
        assert!(new_steward.agent_state().unwrap().private_data().is_none());
        assert!(new_steward
            .agent_state()
            .unwrap()
            .session()
            .encrypted_private
            .is_some());

        new_steward
            .dispatch(parse(r#"/unlock mypass"#).unwrap()[0].clone())
            .await
            .unwrap();
        assert!(new_steward.agent_state().unwrap().private_data().is_some());

        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }
}
