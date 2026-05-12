#[cfg(test)]
mod interpreter_tests {
    use omokoda_core::interpreter::Steward;
    use omokoda_core::parser::parse;
    use omokoda_core::reputation::{tier_for, tool_allowed, tools_for_tier};

    #[test]
    fn birth_creates_agent_at_tier_zero() {
        let mut steward = Steward::new();
        let stmts = parse(r#"birth "luna""#).unwrap();
        steward.dispatch(stmts[0].clone()).unwrap();
        assert_eq!(steward.reputation(), 0.000);
        assert_eq!(steward.tier(), 0);
    }

    #[test]
    fn birth_initializes_structured_agent_state() {
        let mut steward = Steward::new();
        let stmts = parse(r#"birth "luna""#).unwrap();
        steward.dispatch(stmts[0].clone()).unwrap();

        let agent = steward.agent_state().expect("agent exists after birth");
        assert!(agent.id().as_str().starts_with("agent-"));
        assert_eq!(agent.name(), "luna");
        assert!(agent.birth_timestamp() > 0);
        assert_eq!(agent.odu_seed().len(), 32);
        assert_eq!(agent.dna_fingerprint().len(), 86);
        assert!(!agent.odu_identity().mnemonic.is_empty());
        assert_eq!(agent.odu_identity().mnemonic.split_whitespace().count(), 33);
        assert_eq!(agent.reputation(), 0.0);
        assert_eq!(agent.tier(), 0);
    }

    #[test]
    fn think_does_not_produce_receipt() {
        let mut steward = Steward::new();
        parse(r#"birth "luna""#).unwrap().into_iter().for_each(|s| {
            steward.dispatch(s).unwrap();
        });
        let stmts = parse(r#"think "hello world""#).unwrap();
        let result = steward.dispatch(stmts[0].clone()).unwrap();
        assert!(result.receipt.is_none());
    }

    #[test]
    fn think_private_sets_private_mode() {
        let mut steward = Steward::new();
        parse(r#"birth "luna""#).unwrap().into_iter().for_each(|s| {
            steward.dispatch(s).unwrap();
        });
        let stmts = parse(r#"think "secret""#).unwrap();
        let result = steward.dispatch(stmts[0].clone()).unwrap();
        assert!(result.private_mode);
    }

    #[test]
    fn think_publish_sets_public_mode() {
        let mut steward = Steward::new();
        parse(r#"birth "luna""#).unwrap().into_iter().for_each(|s| {
            steward.dispatch(s).unwrap();
        });
        let stmts = parse(r#"think "share this" /publish"#).unwrap();
        let result = steward.dispatch(stmts[0].clone()).unwrap();
        assert!(!result.private_mode);
    }

    #[test]
    fn act_produces_receipt() {
        let mut steward = Steward::new();
        parse(r#"birth "luna""#).unwrap().into_iter().for_each(|s| {
            steward.dispatch(s).unwrap();
        });
        let stmts = parse(r#"act "web_search" "bitcoin""#).unwrap();
        let result = steward.dispatch(stmts[0].clone()).unwrap();
        assert!(result.receipt.is_some());
        let receipt = result.receipt.unwrap();
        assert!(!receipt.dry_run);
        assert!(!receipt.receipt_id.is_empty());
    }

    #[test]
    fn act_increases_reputation_via_dynamic_formula() {
        let mut steward = Steward::new();
        parse(r#"birth "luna""#).unwrap().into_iter().for_each(|s| {
            steward.dispatch(s).unwrap();
        });
        let before = steward.reputation();
        let stmts = parse(r#"act "web_search" "query""#).unwrap();
        steward.dispatch(stmts[0].clone()).unwrap();
        let after = steward.reputation();
        assert!(after > before);
        // At rep ~0, gain should be close to ACT_TIER_0 base (0.040)
        // not a flat +2
        assert!(after - before < 0.1);
        assert!(after - before > 0.0);
    }

    #[test]
    fn reputation_gain_decreases_as_rep_grows() {
        let mut steward = Steward::new();
        parse(r#"birth "luna""#).unwrap().into_iter().for_each(|s| {
            steward.dispatch(s).unwrap();
        });
        // Gain at low rep
        let stmts = parse(r#"act "web_search" "q""#).unwrap();
        steward.dispatch(stmts[0].clone()).unwrap();
        let gain_low = steward.reputation();
        // Manually set rep to 50 to simulate higher reputation
        steward.set_reputation_for_test(50.0);
        let before_high = steward.reputation();
        let stmts2 = parse(r#"act "web_search" "q""#).unwrap();
        steward.dispatch(stmts2[0].clone()).unwrap();
        let gain_high = steward.reputation() - before_high;
        assert!(gain_low > gain_high);
    }

    #[test]
    fn act_rejected_for_tool_above_current_tier() {
        let mut steward = Steward::new();
        parse(r#"birth "luna""#).unwrap().into_iter().for_each(|s| {
            steward.dispatch(s).unwrap();
        });
        // agent_orchestration requires Tier 4 (rep >= 80)
        // new agent is Tier 0
        let stmts = parse(r#"act "agent_orchestration" "task""#).unwrap();
        let result = steward.dispatch(stmts[0].clone());
        assert!(result.is_err());
    }

    #[test]
    fn reputation_decay_on_inactivity() {
        let mut steward = Steward::new();
        parse(r#"birth "luna""#).unwrap().into_iter().for_each(|s| {
            steward.dispatch(s).unwrap();
        });
        steward.set_reputation_for_test(10.0);
        steward.apply_daily_decay(1); // 1 day inactive
        assert!(steward.reputation() < 10.0);
        assert!((steward.reputation() - 9.992).abs() < 0.001);
    }

    #[test]
    fn reputation_cannot_go_below_zero() {
        let mut steward = Steward::new();
        parse(r#"birth "luna""#).unwrap().into_iter().for_each(|s| {
            steward.dispatch(s).unwrap();
        });
        steward.set_reputation_for_test(0.001);
        steward.apply_daily_decay(100); // massive inactivity
        assert_eq!(steward.reputation(), 0.000);
    }

    #[test]
    fn multi_statement_executes_in_order() {
        let mut steward = Steward::new();
        let input = r#"birth "luna"think "hello"act "web_search" "query""#;
        let stmts = parse(input).unwrap();
        assert_eq!(stmts.len(), 3);
        for stmt in stmts {
            steward.dispatch(stmt).unwrap();
        }
        // Reputation must have increased (from the act)
        assert!(steward.reputation() > 0.0);
    }

    #[test]
    fn steward_state_persists_between_dispatches() {
        let mut steward = Steward::new();
        parse(r#"birth "luna""#).unwrap().into_iter().for_each(|s| {
            steward.dispatch(s).unwrap();
        });
        parse(r#"act "web_search" "first""#)
            .unwrap()
            .into_iter()
            .for_each(|s| {
                steward.dispatch(s).unwrap();
            });
        let rep_after_first = steward.reputation();
        parse(r#"act "web_search" "second""#)
            .unwrap()
            .into_iter()
            .for_each(|s| {
                steward.dispatch(s).unwrap();
            });
        let rep_after_second = steward.reputation();
        assert!(rep_after_second > rep_after_first);
    }

    #[test]
    fn reputation_tier_boundaries_match_frozen_spec() {
        assert_eq!(tier_for(0.000), 0);
        assert_eq!(tier_for(20.000), 0);
        assert_eq!(tier_for(20.001), 1);
        assert_eq!(tier_for(39.999), 1);
        assert_eq!(tier_for(40.000), 1);
        assert_eq!(tier_for(40.001), 2);
        assert_eq!(tier_for(59.999), 2);
        assert_eq!(tier_for(60.000), 2);
        assert_eq!(tier_for(60.001), 3);
        assert_eq!(tier_for(79.999), 3);
        assert_eq!(tier_for(80.000), 3);
        assert_eq!(tier_for(80.001), 4);
        assert_eq!(tier_for(99.999), 4);
        assert_eq!(tier_for(100.000), 5);
    }

    #[test]
    fn act_returns_tool_output() {
        let mut steward = Steward::new();
        parse(r#"birth "luna""#).unwrap().into_iter().for_each(|s| {
            steward.dispatch(s).unwrap();
        });
        let stmts = parse(r#"act "read_file" "config.json""#).unwrap();
        let result = steward.dispatch(stmts[0].clone()).unwrap();
        assert!(result.tool_output.is_some());
        assert_eq!(result.tool_output.unwrap(), "contents of config.json");
    }

    #[test]
    fn tier_tool_unlocks_are_cumulative() {
        let tier_0 = tools_for_tier(0);
        assert!(tier_0.contains(&"web_search".to_string()));
        assert!(tier_0.contains(&"read_file".to_string()));
        assert!(tier_0.contains(&"glob".to_string()));
        assert!(tier_0.contains(&"grep".to_string()));
        assert!(tool_allowed(1, "image_gen_basic"));
        assert!(!tool_allowed(1, "code_runner"));
        assert!(tool_allowed(2, "code_runner"));
        assert!(!tool_allowed(2, "data_analysis"));
        assert!(tool_allowed(3, "data_analysis"));
        assert!(tool_allowed(3, "api_connect"));
        assert!(!tool_allowed(3, "agent_orchestration"));
        assert!(tool_allowed(4, "agent_orchestration"));
        assert!(!tool_allowed(4, "self_modification"));
        assert!(tool_allowed(5, "self_modification"));
        assert!(tool_allowed(5, "multi_agent_fabric"));
    }
}
