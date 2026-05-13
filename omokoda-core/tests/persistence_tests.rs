#[cfg(test)]
mod persistence_tests {
    use omokoda_core::interpreter::Steward;
    use omokoda_core::parser::parse;
    use std::path::PathBuf;

    #[tokio::test]
    async fn auto_save_creates_file_on_birth() {
        let mut steward = Steward::new();
        steward.dispatch(parse(r#"birth "luna""#).unwrap()[0].clone()).await.unwrap();
        
        let agent_id = steward.agent_state().unwrap().id().to_string();
        let path = PathBuf::from("sessions").join(format!("{}.json", agent_id));
        
        assert!(path.exists());
        
        // Cleanup
        let _ = std::fs::remove_file(path);
    }

    #[tokio::test]
    async fn persistence_roundtrip() {
        let mut steward = Steward::new();
        steward.dispatch(parse(r#"birth "luna""#).unwrap()[0].clone()).await.unwrap();
        
        let agent_id = steward.agent_state().unwrap().id().clone();
        let dna = steward.agent_state().unwrap().dna_fingerprint().to_string();
        
        // Start a new steward and load the agent
        let mut new_steward = Steward::new();
        new_steward.load_agent(&agent_id).unwrap();
        
        assert_eq!(new_steward.agent_state().unwrap().name(), "luna");
        assert_eq!(new_steward.agent_state().unwrap().dna_fingerprint(), dna);
        
        // Cleanup
        let path = PathBuf::from("sessions").join(format!("{}.json", agent_id));
        let _ = std::fs::remove_file(path);
    }

    #[tokio::test]
    async fn auto_save_updates_reputation() {
        let mut steward = Steward::new();
        steward.dispatch(parse(r#"birth "luna""#).unwrap()[0].clone()).await.unwrap();
        
        let agent_id = steward.agent_state().unwrap().id().clone();
        
        // Manually update reputation and check if it's saved
        steward.set_reputation_for_test(10.0);
        
        let mut new_steward = Steward::new();
        new_steward.load_agent(&agent_id).unwrap();
        assert_eq!(new_steward.reputation(), 10.0);
        
        // Cleanup
        let path = PathBuf::from("sessions").join(format!("{}.json", agent_id));
        let _ = std::fs::remove_file(path);
    }
}
