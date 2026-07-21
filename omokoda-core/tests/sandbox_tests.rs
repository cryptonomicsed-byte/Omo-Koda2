#[cfg(test)]
mod sandbox_tests {
    use omokoda_core::tools::ToolRegistry;
    use std::fs;
    use wat::parse_str;

    #[tokio::test]
    async fn wasm_tool_executes_simple_module_in_sandbox() {
        std::env::set_var("OMOKODA_ENABLE_WASM", "1");
        let wasm_bytes = parse_str(
            r#"(module
  (func (export "main")
    nop
  )
)"#,
        )
        .unwrap();

        fs::write("test_simple.wasm", &wasm_bytes).unwrap();
        let registry = ToolRegistry::new();

        let ctx = omokoda_core::tools::ExecutionContext {
            agent_id: omokoda_core::identity::AgentId::from_str("agent-1"),
            name: "luna".to_string(),
            tier: 2,
            reputation: 0.0,
            odu_identity: omokoda_core::identity::odu::OduIdentity {
                primary_index: 0,
                mnemonic: "".into(),
            },
            workspace_root: std::env::current_dir().unwrap(),
            sandbox_mode: true,
        };

        let policy = omokoda_core::permissions::PermissionPolicy::default_steward_policy(
            omokoda_core::permissions::PermissionMode::DangerFullAccess,
        );
        let result = registry
            .execute("wasm", "test_simple.wasm", ctx, &policy, None)
            .await;
        assert!(result.is_ok(), "WASM execution failed: {:?}", result.err());
        assert_eq!(result.unwrap().0, "WASM execution completed");

        fs::remove_file("test_simple.wasm").unwrap();
    }

    #[tokio::test]
    async fn wasm_tool_rejects_outside_workspace_paths() {
        std::env::set_var("OMOKODA_ENABLE_WASM", "1");
        let registry = ToolRegistry::new();
        let policy = omokoda_core::permissions::PermissionPolicy::default_steward_policy(
            omokoda_core::permissions::PermissionMode::DangerFullAccess,
        );
        let ctx = omokoda_core::tools::ExecutionContext {
            agent_id: omokoda_core::identity::AgentId::from_str("agent-1"),
            name: "luna".to_string(),
            tier: 2,
            reputation: 0.0,
            odu_identity: omokoda_core::identity::odu::OduIdentity {
                primary_index: 0,
                mnemonic: "".into(),
            },
            workspace_root: std::env::current_dir().unwrap(),
            sandbox_mode: true,
        };

        let result = registry
            .execute("wasm", "../secret.wasm", ctx, &policy, None)
            .await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("module path must be relative and within workspace"));
    }
}
