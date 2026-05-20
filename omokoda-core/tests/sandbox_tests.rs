#[cfg(test)]
mod sandbox_tests {
    use omokoda_core::tools::ToolRegistry;
    use std::fs;
    use wat::parse_str;

    #[tokio::test]
    async fn wasm_tool_executes_simple_module_in_sandbox() {
        let wasm_bytes = parse_str(
            r#"(module
  (import "wasi_snapshot_preview1" "fd_write" (func $fd_write (param i32 i32 i32 i32) (result i32)))
  (import "wasi_snapshot_preview1" "proc_exit" (func $proc_exit (param i32)))
  (memory 1)
  (export "memory" (memory 0))
  (data (i32.const 8) "Hello, WASM!\n")
  (func $main (result i32)
    (call $fd_write (i32.const 1) (i32.const 8) (i32.const 12) (i32.const 0))
    (call $proc_exit (i32.const 0))
    (i32.const 0)
  )
  (start $main)
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
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0, "WASM execution succeeded");

        fs::remove_file("test_simple.wasm").unwrap();
    }

    #[tokio::test]
    async fn wasm_tool_rejects_outside_workspace_paths() {
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
