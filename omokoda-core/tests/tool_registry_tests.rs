#[cfg(test)]
mod tool_registry_tests {
    use omokoda_core::tools::ToolRegistry;
    use omokoda_core::permissions::{PermissionPolicy, PermissionMode};
    use std::fs;

    #[tokio::test]
    async fn read_file_tool_basic() {
        let registry = ToolRegistry::new();
        let policy = PermissionPolicy::default_steward_policy(PermissionMode::WorkspaceWrite);
        let test_file = "test_read_file.txt";
        fs::write(test_file, "hello world").unwrap();

        let result = registry
            .execute("read_file", test_file, false, 0, &policy, None)
            .await
            .unwrap();
        assert!(result.contains("hello world"));
        assert!(result.contains("\"file\":"));

        fs::remove_file(test_file).unwrap();
    }

    #[tokio::test]
    async fn glob_tool_basic() {
        let registry = ToolRegistry::new();
        let policy = PermissionPolicy::default_steward_policy(PermissionMode::WorkspaceWrite);
        fs::create_dir_all("test_glob_dir").unwrap();
        fs::write("test_glob_dir/a.txt", "a").unwrap();
        fs::write("test_glob_dir/b.txt", "b").unwrap();

        let result = registry
            .execute("glob", "test_glob_dir/*.txt", false, 0, &policy, None)
            .await
            .unwrap();
        assert!(result.contains("test_glob_dir/a.txt"));
        assert!(result.contains("test_glob_dir/b.txt"));
        assert!(result.contains("\"filenames\":"));

        fs::remove_dir_all("test_glob_dir").unwrap();
    }

    #[tokio::test]
    async fn grep_tool_basic() {
        let registry = ToolRegistry::new();
        let policy = PermissionPolicy::default_steward_policy(PermissionMode::WorkspaceWrite);
        let test_file = "test_grep.txt";
        fs::write(test_file, "line 1\nline 2 with target\nline 3").unwrap();

        let grep_input = serde_json::json!({
            "pattern": "target",
            "path": ".",
            "glob": "**/test_grep.txt",
            "output_mode": "content"
        });

        let result = registry
            .execute("grep", &grep_input.to_string(), false, 0, &policy, None)
            .await
            .unwrap();
        assert!(result.contains(":2:line 2 with target"));
        assert!(result.contains("\"content\":"));

        fs::remove_file(test_file).unwrap();
    }

    #[tokio::test]
    async fn tools_enforce_tier_gates() {
        let registry = ToolRegistry::new();
        let policy = PermissionPolicy::default_steward_policy(PermissionMode::DangerFullAccess);

        // bash requires Tier 2
        let result = registry.execute("bash", "ls", false, 0, &policy, None).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("requires tier 2"));

        let result = registry.execute("bash", "ls", false, 2, &policy, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn tools_block_path_traversal() {
        let registry = ToolRegistry::new();
        let policy = PermissionPolicy::default_steward_policy(PermissionMode::WorkspaceWrite);

        let result = registry
            .execute("read_file", "../secrets.txt", false, 0, &policy, None)
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Boundary Violation"));

        let result = registry.execute("glob", "../**/*", false, 0, &policy, None).await;
        assert!(result.is_err());

        let grep_input = serde_json::json!({
            "pattern": "secret",
            "path": "../file.txt"
        });

        let result = registry
            .execute("grep", &grep_input.to_string(), false, 0, &policy, None)
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Boundary Violation"));
    }

    #[tokio::test]
    async fn bash_sandbox_rejects_parent_traversal() {
        let registry = ToolRegistry::new();
        let policy = PermissionPolicy::default_steward_policy(PermissionMode::DangerFullAccess);
        let result = registry.execute("bash", "cd ../ && ls", true, 2, &policy, None).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must not contain '..'"));
    }
}
