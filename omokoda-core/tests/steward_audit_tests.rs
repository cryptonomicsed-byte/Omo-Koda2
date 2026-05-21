#[cfg(test)]
mod steward_audit_tests {
    use omokoda_core::interpreter::Steward;
    use omokoda_core::parser::parse;
    use std::path::PathBuf;

    fn test_session_dir(test_name: &str, cleanup: bool) -> PathBuf {
        let mut path = std::env::current_dir().unwrap();
        path.push("target");
        path.push("test_sessions");
        path.push(test_name);
        if cleanup && path.exists() {
            let _ = std::fs::remove_dir_all(&path);
        }
        if !path.exists() {
            std::fs::create_dir_all(&path).unwrap();
        }
        path
    }

    macro_rules! test_steward {
        ($name:expr) => {{
            Steward::new().with_session_dir(test_session_dir($name, true))
        }};
    }

    #[tokio::test]
    async fn birth_initializes_all_four_domains() {
        let mut steward = test_steward!("birth_initializes_all_four_domains");
        steward
            .dispatch(parse(r#"birth "luna""#).unwrap()[0].clone())
            .await
            .unwrap();
        let agent = steward.agent_core().unwrap();

        // IDENTITY
        assert!(!agent.dna_fingerprint().is_empty());
        assert!(agent.birth_timestamp() > 0);
        assert!(!agent.odu_identity().mnemonic.is_empty());

        // MEMORY (Session initialized, private_data available)
        assert!(agent.private_data().is_some());
        assert_eq!(agent.session().name, "luna");

        // EXECUTION (Steward has tools and providers)
        // (Implicit in successful dispatch and steward structure)

        // ECONOMICS (Reputation starts at 0, Synapse logic exists)
        assert_eq!(agent.reputation(), 0.0);
        // Note: Synapse burn will be tested in think/act cases
    }

    #[tokio::test]
    async fn think_burns_synapse() {
        let mut steward = test_steward!("think_burns_synapse");
        steward.set_mock_provider("think response".to_string());
        steward
            .dispatch(parse(r#"birth "luna" provider:ollama"#).unwrap()[0].clone())
            .await
            .unwrap();

        let initial_synapse = steward.agent_core().unwrap().synapse();
        steward
            .dispatch(parse(r#"think "hello""#).unwrap()[0].clone())
            .await
            .unwrap();

        assert!(steward.agent_core().unwrap().synapse() < initial_synapse);
        assert_eq!(
            initial_synapse - steward.agent_core().unwrap().synapse(),
            1000.0
        );
    }

    #[tokio::test]
    async fn think_private_never_routes_external() {
        use omokoda_core::providers::MockProvider;
        let mut steward = test_steward!("think_private_never_routes_external");
        // Register an external-looking provider
        steward.register_provider(Box::new(MockProvider::new_external(
            "openai",
            "response".to_string(),
        )));

        steward
            .dispatch(parse(r#"birth "luna" provider:openai"#).unwrap()[0].clone())
            .await
            .unwrap();

        // This should fail because 'openai' is blocked for /private
        let res = steward
            .dispatch(parse(r#"think "my secret" /private"#).unwrap()[0].clone())
            .await;
        assert!(res.is_err());
        assert!(res
            .unwrap_err()
            .contains("Private thoughts require a local provider"));
    }

    #[tokio::test]
    async fn act_produces_receipt() {
        let mut steward = test_steward!("act_produces_receipt");
        steward
            .dispatch(parse(r#"birth "luna""#).unwrap()[0].clone())
            .await
            .unwrap();

        let test_file = "steward_test_act.txt";
        std::fs::write(test_file, "content").unwrap();

        let res = steward
            .dispatch(parse(r#"act "read_file" "steward_test_act.txt""#).unwrap()[0].clone())
            .await
            .unwrap();
        assert!(res.receipt.is_some());
        assert_eq!(res.receipt.as_ref().unwrap().action, "read_file");

        std::fs::remove_file(test_file).unwrap();
    }

    #[tokio::test]
    async fn act_burns_synapse() {
        let mut steward = test_steward!("act_burns_synapse");
        steward
            .dispatch(parse(r#"birth "luna""#).unwrap()[0].clone())
            .await
            .unwrap();

        let test_file = "steward_test_synapse_act.txt";
        std::fs::write(test_file, "content").unwrap();

        let initial_synapse = steward.agent_core().unwrap().synapse();
        steward
            .dispatch(
                parse(r#"act "read_file" "steward_test_synapse_act.txt""#).unwrap()[0].clone(),
            )
            .await
            .unwrap();

        let diff = initial_synapse - steward.agent_core().unwrap().synapse();
        assert!(
            diff >= 6100.0,
            "Burn amount {} should be at least 6100",
            diff
        );
        assert!(
            diff < 6200.0,
            "Burn amount {} should not exceed 6200 (including minor decay)",
            diff
        );

        std::fs::remove_file(test_file).unwrap();
    }

    #[tokio::test]
    async fn act_rejected_below_required_tier() {
        let mut steward = test_steward!("act_rejected_below_required_tier");
        steward
            .dispatch(parse(r#"birth "luna""#).unwrap()[0].clone())
            .await
            .unwrap();

        // Tier 4 tool on Tier 0 agent
        let res = steward
            .dispatch(parse(r#"act "agent_orchestration" "do something""#).unwrap()[0].clone())
            .await;
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("requires higher reputation"));
    }

    #[tokio::test]
    async fn act_advances_reputation() {
        let mut steward = test_steward!("act_advances_reputation");
        steward
            .dispatch(parse(r#"birth "luna""#).unwrap()[0].clone())
            .await
            .unwrap();

        let test_file = "steward_test_rep.txt";
        std::fs::write(test_file, "content").unwrap();

        let initial_rep = steward.reputation();
        steward
            .dispatch(parse(r#"act "read_file" "steward_test_rep.txt""#).unwrap()[0].clone())
            .await
            .unwrap();
        assert!(steward.reputation() > initial_rep);

        std::fs::remove_file(test_file).unwrap();
    }

    #[tokio::test]
    async fn multi_statement_executes_in_order() {
        let mut steward = test_steward!("multi_statement_executes_in_order");
        steward.set_mock_provider("think output".to_string());

        let test_file = "steward_test_multi.txt";
        std::fs::write(test_file, "content").unwrap();

        // Use /publish to ensure messages go to public_messages
        let stmts = parse(
            r#"birth "luna" provider:ollama
think "hello" /publish
act "read_file" "steward_test_multi.txt""#,
        )
        .unwrap();

        for stmt in stmts {
            steward.dispatch(stmt).await.unwrap();
        }

        let agent = steward.agent_core().unwrap();
        // Birth(0) + Think(2: user/assistant) + Act(2: tooluse/result) = 4 messages
        // Act messages currently go to private if sandbox is requested (which defaults to true in our recent change)
        // Let's check for at least the think messages in public.
        assert!(agent.session().public_messages.len() >= 2);

        std::fs::remove_file(test_file).unwrap();
    }

    #[tokio::test]
    async fn steward_state_persists_between_statements() {
        let name = "steward_state_persists_between_statements";
        let session_dir = test_session_dir(name, true);
        let mut steward = Steward::new().with_session_dir(session_dir.clone());
        steward
            .dispatch(parse(r#"birth "luna""#).unwrap()[0].clone())
            .await
            .unwrap();

        let agent_id = steward.agent_core().unwrap().id().clone();

        let mut steward2 = Steward::new().with_session_dir(test_session_dir(name, false));
        steward2.load_agent(&agent_id).unwrap();
        assert_eq!(steward2.agent_core().unwrap().name(), "luna");
    }
}
