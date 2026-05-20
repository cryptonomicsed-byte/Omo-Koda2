#[cfg(test)]
mod e2e_tests {
    use omokoda_core::{parse, Steward};
    use std::fs;
    use std::path::PathBuf;
    use wat::parse_str;

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
    async fn e2e_birth_think_and_wasm_act_flow() {
        let mut steward = test_steward!("e2e_birth_think_and_wasm_act_flow");
        steward.set_mock_provider("e2e thought".to_string());
        steward.set_permission_mode(omokoda_core::permissions::PermissionMode::Allow);

        let stmts = parse(r#"birth "koda" provider:ollama sandbox:true"#).unwrap();
        steward.dispatch(stmts[0].clone()).await.unwrap();
        steward.set_reputation_for_test(50.0);

        let think_stmts = parse(r#"think "what is two plus two?""#).unwrap();
        let think_result = steward.dispatch(think_stmts[0].clone()).await.unwrap();
        assert_eq!(think_result.tool_output.unwrap(), "e2e thought");

        let wasm_bytes = parse_str(
            r#"(module
  (func (export "_start"))
)"#,
        )
        .unwrap();

        fs::write("test_e2e.wasm", &wasm_bytes).unwrap();

        let act_stmts = parse(r#"act "wasm" "test_e2e.wasm" /sandbox"#).unwrap();
        let act_result = steward.dispatch(act_stmts[0].clone()).await.unwrap();
        assert_eq!(act_result.tool_output.unwrap(), "WASM execution completed");

        fs::remove_file("test_e2e.wasm").unwrap();
    }
}
