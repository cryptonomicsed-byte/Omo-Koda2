#[cfg(test)]
mod sandbox_tests {
    use omokoda_core::tools::ToolRegistry;
    use std::fs;
    use wat::parse_str;

    #[tokio::test]
    async fn wasm_tool_executes_simple_module_in_sandbox() {
        let wasm_bytes = parse_str(r#"(module
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
)"#).unwrap();

        fs::write("test_simple.wasm", &wasm_bytes).unwrap();
        let registry = ToolRegistry::new();

        let result = registry.execute("wasm", "test_simple.wasm", true, 2).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "WASM execution succeeded");

        fs::remove_file("test_simple.wasm").unwrap();
    }

    #[tokio::test]
    async fn wasm_tool_rejects_outside_workspace_paths() {
        let registry = ToolRegistry::new();
        let result = registry.execute("wasm", "../secret.wasm", true, 2).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("relative and within workspace"));
    }
}
