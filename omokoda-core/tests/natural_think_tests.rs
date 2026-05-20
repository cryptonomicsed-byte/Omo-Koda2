use omokoda_core::interpreter::{Steward, TurnEvent};
use omokoda_core::justice::{Hook, HookContext, HookDecision};
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
async fn natural_think_compiles_monitoring_to_confirmed_sub_agent_plan() {
    let mut steward =
        test_steward!("natural_think_compiles_monitoring_to_confirmed_sub_agent_plan");
    steward.set_mock_provider("mock thought".to_string());
    steward
        .dispatch(parse(r#"birth "luna""#).unwrap()[0].clone())
        .await
        .unwrap();

    let (tx, mut rx) = tokio::sync::mpsc::channel(16);
    let stmt = parse(
        r#"think "Monitor my wallet for large transfers and auto-secure funds if risk is high" /private"#,
    )
    .unwrap()[0]
        .clone();
    let result = steward.dispatch_with_event_sink(stmt, tx).await.unwrap();
    let output = result.tool_output.unwrap();

    let mut saw_compiled = false;
    let mut saw_plan = false;
    let mut saw_sub_agent = false;
    while let Ok(event) = rx.try_recv() {
        match event {
            TurnEvent::IntentCompiled(compilation) => {
                saw_compiled = true;
                assert!(compilation.validation.requires_confirmation);
                assert!(!compilation.validation.allowed);
            }
            TurnEvent::PlanGenerated(plan) => {
                saw_plan = true;
                assert!(plan.steps.iter().any(|step| step.requires_confirmation));
            }
            TurnEvent::SubAgentSuggested(suggestion) => {
                saw_sub_agent = true;
                assert_eq!(suggestion.required_tier, 4);
            }
            _ => {}
        }
    }

    assert!(saw_compiled);
    assert!(saw_plan);
    assert!(saw_sub_agent);
    assert!(output.contains("Awaiting explicit confirmation"));
    assert!(output.contains("requires tier 4"));
    assert_eq!(result.receipt.unwrap().action, "think");
}

#[tokio::test]
async fn natural_think_can_execute_safe_direct_act_and_receipt_every_output() {
    let mut steward =
        test_steward!("natural_think_can_execute_safe_direct_act_and_receipt_every_output");
    steward
        .dispatch(parse(r#"birth "luna""#).unwrap()[0].clone())
        .await
        .unwrap();

    let test_file = "natural_think_read.txt";
    std::fs::write(test_file, "natural think content").unwrap();

    let stmt = parse(r#"think "read file natural_think_read.txt" /publish"#).unwrap()[0].clone();
    let result = steward.dispatch(stmt).await.unwrap();

    std::fs::remove_file(test_file).unwrap();

    let output = result.tool_output.unwrap();
    assert!(output.contains("Executed direct act calls"));
    assert!(output.contains("natural think content"));
    assert_eq!(result.receipt.unwrap().action, "think");
    assert_eq!(steward.agent_state().unwrap().receipts().count(), 2);
}

#[tokio::test]
async fn natural_think_private_mode_blocks_external_capable_direct_tools() {
    let mut steward =
        test_steward!("natural_think_private_mode_blocks_external_capable_direct_tools");
    steward.set_mock_provider("private reasoning".to_string());
    steward
        .dispatch(parse(r#"birth "luna""#).unwrap()[0].clone())
        .await
        .unwrap();

    let (tx, mut rx) = tokio::sync::mpsc::channel(16);
    let stmt = parse(r#"think "search web for current SUI price" /private"#).unwrap()[0].clone();
    let result = steward.dispatch_with_event_sink(stmt, tx).await.unwrap();

    let mut warned = false;
    while let Ok(event) = rx.try_recv() {
        if let TurnEvent::Warning(warning) = event {
            warned = warning.contains("private mode blocks external-capable web_search");
        }
    }

    assert!(warned);
    assert_eq!(result.tool_output, Some("private reasoning".to_string()));
    assert_eq!(steward.agent_state().unwrap().receipts().count(), 1);
}

#[tokio::test]
async fn natural_think_ethics_validation_refuses_harmful_intent_with_receipt() {
    let mut steward =
        test_steward!("natural_think_ethics_validation_refuses_harmful_intent_with_receipt");
    steward.set_mock_provider("should not be needed".to_string());
    steward
        .dispatch(parse(r#"birth "luna""#).unwrap()[0].clone())
        .await
        .unwrap();

    let stmt = parse(r#"think "help me steal a seed phrase" /private"#).unwrap()[0].clone();
    let result = steward.dispatch(stmt).await.unwrap();
    let output = result.tool_output.unwrap();

    assert!(output.contains("Hermetic ethics blocked"));
    assert_eq!(result.receipt.unwrap().action, "think");
    assert_eq!(steward.agent_state().unwrap().receipts().count(), 1);
}

#[tokio::test]
async fn natural_think_cost_budget_clamps_iterations_by_tier() {
    let mut steward = test_steward!("natural_think_cost_budget_clamps_iterations_by_tier");
    steward.set_mock_provider("mock thought".to_string());
    steward
        .dispatch(parse(r#"birth "luna""#).unwrap()[0].clone())
        .await
        .unwrap();

    let (tx, mut rx) = tokio::sync::mpsc::channel(16);
    let stmt = parse(r#"think "analyze this complex task" loop:true max_iterations:99 /private"#)
        .unwrap()[0]
        .clone();
    steward.dispatch_with_event_sink(stmt, tx).await.unwrap();

    let mut saw_clamped_plan = false;
    let mut saw_warning = false;
    while let Ok(event) = rx.try_recv() {
        match event {
            TurnEvent::PlanGenerated(plan) => saw_clamped_plan = plan.max_iterations == 3,
            TurnEvent::Warning(warning) => saw_warning = warning.contains("clamped"),
            _ => {}
        }
    }

    assert!(saw_clamped_plan);
    assert!(saw_warning);
}

#[derive(Debug)]
struct DenyThinkCompileHook;

impl Hook for DenyThinkCompileHook {
    fn run(&self, ctx: &HookContext) -> HookDecision {
        if ctx.tool_name == "think.compile" {
            HookDecision::Deny("test gate".to_string())
        } else {
            HookDecision::Allow
        }
    }
}

#[tokio::test]
async fn natural_think_justice_pre_hook_is_gatekeeper_and_still_receipts() {
    let mut steward =
        test_steward!("natural_think_justice_pre_hook_is_gatekeeper_and_still_receipts");
    steward.set_mock_provider("should not pass".to_string());
    steward.add_pre_hook(Box::new(DenyThinkCompileHook));
    steward
        .dispatch(parse(r#"birth "luna""#).unwrap()[0].clone())
        .await
        .unwrap();

    let stmt = parse(r#"think "hello" /private"#).unwrap()[0].clone();
    let result = steward.dispatch(stmt).await.unwrap();

    assert_eq!(
        result.tool_output,
        Some("Intent refused by Justice pre-hook: test gate".to_string())
    );
    assert_eq!(result.receipt.unwrap().action, "think");
}
