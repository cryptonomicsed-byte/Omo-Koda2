#[cfg(test)]
mod tool_registry_tests {
    use omokoda_core::tools::ToolRegistry;
    use std::fs;

    #[tokio::test]
    async fn read_file_tool_basic() {
        let registry = ToolRegistry::new();
        let test_file = "test_read_file.txt";
        fs::write(test_file, "hello world").unwrap();
        
        let result = registry.execute("read_file", test_file, false, 0).await.unwrap();
        assert_eq!(result, "hello world");
        
        fs::remove_file(test_file).unwrap();
    }

    #[tokio::test]
    async fn glob_tool_basic() {
        let registry = ToolRegistry::new();
        fs::create_dir_all("test_glob_dir").unwrap();
        fs::write("test_glob_dir/a.txt", "a").unwrap();
        fs::write("test_glob_dir/b.txt", "b").unwrap();
        
        let result = registry.execute("glob", "test_glob_dir/*.txt", false, 0).await.unwrap();
        assert!(result.contains("test_glob_dir/a.txt"));
        assert!(result.contains("test_glob_dir/b.txt"));
        
        fs::remove_dir_all("test_glob_dir").unwrap();
    }

    #[tokio::test]
    async fn grep_tool_basic() {
        let registry = ToolRegistry::new();
        let test_file = "test_grep.txt";
        fs::write(test_file, "line 1\nline 2 with target\nline 3").unwrap();
        
        let result = registry.execute("grep", "target test_grep.txt", false, 0).await.unwrap();
        assert!(result.contains("2: line 2 with target"));
        assert!(!result.contains("line 1"));
        
        fs::remove_file(test_file).unwrap();
    }

    #[tokio::test]
    async fn tools_enforce_tier_gates() {
        let registry = ToolRegistry::new();
        
        // bash requires Tier 2
        let result = registry.execute("bash", "ls", false, 0).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("requires tier 2"));
        
        let result = registry.execute("bash", "ls", false, 2).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn tools_block_path_traversal() {
        let registry = ToolRegistry::new();
        
        let result = registry.execute("read_file", "../secrets.txt", false, 0).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("no .. allowed"));
        
        let result = registry.execute("glob", "../**/*", false, 0).await;
        assert!(result.is_err());
        
        let result = registry.execute("grep", "secret ../file.txt", false, 0).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn bash_sandbox_rejects_parent_traversal() {
        let registry = ToolRegistry::new();
        let result = registry.execute("bash", "cd ../ && ls", true, 2).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must not contain '..'"));
    }
}
