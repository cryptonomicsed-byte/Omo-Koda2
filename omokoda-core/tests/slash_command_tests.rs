#[cfg(test)]
mod slash_command_tests {
    use omokoda_core::interpreter::Steward;
    use omokoda_core::parser::parse;

    #[tokio::test]
    async fn slash_status_returns_agent_info() {
        let mut steward = Steward::new();
        steward.dispatch(parse(r#"birth "luna""#).unwrap()[0].clone()).await.unwrap();
        
        let result = steward.dispatch(parse("/status").unwrap()[0].clone()).await.unwrap();
        let output = result.tool_output.unwrap();
        assert!(output.contains("Agent ID: agent-"));
        assert!(output.contains("Name: luna"));
        assert!(output.contains("Reputation: 0.000"));
        assert!(output.contains("Tier: 0"));
    }

    #[tokio::test]
    async fn slash_tools_lists_tier_zero_tools() {
        let mut steward = Steward::new();
        steward.dispatch(parse(r#"birth "luna""#).unwrap()[0].clone()).await.unwrap();
        
        let result = steward.dispatch(parse("/tools").unwrap()[0].clone()).await.unwrap();
        let output = result.tool_output.unwrap();
        assert!(output.contains("Allowed tools for Tier 0:"));
        assert!(output.contains("- web_search"));
        assert!(output.contains("- read_file"));
    }

    #[tokio::test]
    async fn slash_configure_updates_config() {
        let mut steward = Steward::new();
        steward.set_mock_provider("mock response".to_string());
        steward.dispatch(parse(r#"birth "luna""#).unwrap()[0].clone()).await.unwrap();
        
        let result = steward.dispatch(parse("/configure provider:mock").unwrap()[0].clone()).await.unwrap();
        assert!(result.tool_output.unwrap().contains("Configured provider to mock"));
        
        let agent = steward.agent_state().unwrap();
        assert_eq!(agent.session().config.default_provider, "mock");
    }

    #[tokio::test]
    async fn slash_help_returns_help_text() {
        let mut steward = Steward::new();
        let result = steward.dispatch(parse("/help").unwrap()[0].clone()).await.unwrap();
        let output = result.tool_output.unwrap();
        assert!(output.contains("Omokoda CLI Help:"));
        assert!(output.contains("/status"));
        assert!(output.contains("/tools"));
    }

    #[tokio::test]
    async fn slash_commands_require_birth_except_help() {
        let mut steward = Steward::new();
        
        // Help should work without birth
        assert!(steward.dispatch(parse("/help").unwrap()[0].clone()).await.is_ok());
        
        // Others should fail
        assert!(steward.dispatch(parse("/status").unwrap()[0].clone()).await.is_err());
        assert!(steward.dispatch(parse("/tools").unwrap()[0].clone()).await.is_err());
        assert!(steward.dispatch(parse("/configure x:y").unwrap()[0].clone()).await.is_err());
    }
}
