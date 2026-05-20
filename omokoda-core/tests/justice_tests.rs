#[cfg(test)]
mod justice_tests {
    use omokoda_core::interpreter::Steward;
    use omokoda_core::justice::{ActQuality, JusticeEngine};
    use omokoda_core::parser::parse;
    use std::path::PathBuf;

    macro_rules! test_steward {
        ($name:expr) => {{
            let mut path = std::env::current_dir().unwrap();
            path.push("target");
            path.push("test_sessions");
            path.push($name);
            if path.exists() {
                let _ = std::fs::remove_dir_all(&path);
            }
            std::fs::create_dir_all(&path).unwrap();
            Steward::new().with_session_dir(path)
        }};
    }

    #[tokio::test]
    async fn slashing_ethics_reduces_reputation_by_25_percent() {
        let mut steward = test_steward!("slashing_ethics_reduces_reputation_by_25_percent");
        steward
            .dispatch(parse(r#"birth "luna""#).unwrap()[0].clone())
            .await
            .unwrap();

        steward.set_reputation_for_test(100.0);
        steward.slash_ethics().unwrap();

        assert_eq!(steward.reputation(), 75.0);
    }

    #[tokio::test]
    async fn slashing_budget_reduces_reputation_by_10_percent() {
        let mut steward = test_steward!("slashing_budget_reduces_reputation_by_10_percent");
        steward
            .dispatch(parse(r#"birth "luna""#).unwrap()[0].clone())
            .await
            .unwrap();

        steward.set_reputation_for_test(100.0);
        steward.slash_budget().unwrap();

        assert_eq!(steward.reputation(), 90.0);
    }

    #[test]
    fn quality_evaluation_failed_multiplier_is_negative() {
        let justice = JusticeEngine::new();
        let quality = justice.evaluate_act("Error: failed", true);
        assert_eq!(quality, ActQuality::Failed);
        assert_eq!(quality.multiplier(), -0.5);
    }

    #[tokio::test]
    async fn quality_evaluation_useful_increases_reputation_more_than_basic() {
        let mut steward =
            test_steward!("quality_evaluation_useful_increases_reputation_more_than_basic");
        steward
            .dispatch(parse(r#"birth "luna""#).unwrap()[0].clone())
            .await
            .unwrap();
        steward.ensure_born_mut().unwrap().set_synapse(100_000.0);
 // Boost budget

        // Basic: very short output
        let test_file = "basic.txt";
        std::fs::write(test_file, "short").unwrap();
        steward.set_reputation_for_test(10.0);
        steward
            .dispatch(parse(r#"act "read_file" "basic.txt""#).unwrap()[0].clone())
            .await
            .unwrap();
        let gain_basic = steward.reputation() - 10.0;
        std::fs::remove_file(test_file).unwrap();

        // Useful: > 100 chars
        let useful_content = "A".repeat(150);
        let test_file2 = "useful.txt";
        std::fs::write(test_file2, &useful_content).unwrap();
        steward.clear_cooldowns();
        steward.set_reputation_for_test(10.0);
        steward
            .dispatch(parse(r#"act "read_file" "useful.txt""#).unwrap()[0].clone())
            .await
            .unwrap();
        let gain_useful = steward.reputation() - 10.0;
        std::fs::remove_file(test_file2).unwrap();

        assert!(gain_useful > gain_basic);
    }

    #[test]
    fn hook_runner_pre_act_denial() {
        use omokoda_core::justice::{HookContext, HookDecision, HookRunner, ReputationGate};
        let mut runner = HookRunner::new();
        runner.pre_act.push(Box::new(ReputationGate {
            min_reputation: 50.0,
        }));

        let ctx = HookContext {
            tool_name: "test_tool".to_string(),
            input: "input".to_string(),
            output: None,
            reputation: 10.0,
            tier: 0,
        };

        let bus = omokoda_core::bus::SovereignEventBus::default();
        match runner.run_pre(&ctx, &bus) {
            HookDecision::Deny(reason) => assert!(reason.contains("Reputation too low")),
            _ => panic!("Should have been denied"),
        }

        let ctx_high = HookContext {
            tool_name: "test_tool".to_string(),
            input: "input".to_string(),
            output: None,
            reputation: 60.0,
            tier: 2,
        };
        assert!(matches!(runner.run_pre(&ctx_high, &bus), HookDecision::Allow));
    }

    #[tokio::test]
    async fn steward_act_respects_hook_denial() {
        use omokoda_core::justice::ReputationGate;
        let mut steward = test_steward!("steward_act_respects_hook_denial");
        steward
            .dispatch(parse(r#"birth "luna""#).unwrap()[0].clone())
            .await
            .unwrap();
        steward.ensure_born_mut().unwrap().set_synapse(100_000.0);
 // Boost budget
        steward.set_reputation_for_test(10.0);

        steward.add_pre_hook(Box::new(ReputationGate {
            min_reputation: 50.0,
        }));

        let res = steward
            .dispatch(parse(r#"act "read_file" "basic.txt""#).unwrap()[0].clone())
            .await;
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Hook denied execution"));
    }
}
