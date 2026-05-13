#[cfg(test)]
mod provider_tests {
    use httpmock::Method::POST;
    use httpmock::MockServer;
    use omokoda_core::providers::{AnthropicProvider, LlmProvider, OpenAIProvider, ProviderRegistry};
    use omokoda_core::session::ConversationMessage;
    use serde_json::json;

    #[tokio::test]
    async fn openai_provider_submits_chat_completion() {
        let server = MockServer::start_async().await;
        let mock = server.mock_async(|when, then| {
            when.method(POST)
                .path("/v1/chat/completions")
                .header("authorization", "Bearer test-key")
                .body_contains("gpt-4o-mini")
                .body_contains("hello there");
            then.status(200)
                .json_body(json!({
                    "choices": [{"message": {"content": "openai reply"}}]
                }));
        })
        .await;

        let provider = OpenAIProvider::new(
            "test-key".to_string(),
            Some("gpt-4o-mini".to_string()),
            Some(server.url("")),
        );

        let response = provider.generate("hello there", &[]).await.unwrap();
        assert_eq!(response, "openai reply");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn anthropic_provider_submits_completion_payload() {
        let server = MockServer::start_async().await;
        let mock = server.mock_async(|when, then| {
            when.method(POST)
                .path("/v1/complete")
                .header("authorization", "Bearer test-key")
                .body_contains("Human: hello there")
                .body_contains("claude-3.0");
            then.status(200)
                .json_body(json!({
                    "completion": "anthropic reply"
                }));
        })
        .await;

        let provider = AnthropicProvider::new(
            "test-key".to_string(),
            Some("claude-3.0".to_string()),
            Some(server.url("")),
        );

        let response = provider.generate("hello there", &[]).await.unwrap();
        assert_eq!(response, "anthropic reply");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn registry_register_openai_and_anthropic_providers() {
        let server = MockServer::start_async().await;
        let openai_mock = server.mock_async(|when, then| {
            when.method(POST)
                .path("/v1/chat/completions");
            then.status(200)
                .json_body(json!({"choices": [{"message": {"content": "openai reply"}}]}));
        })
        .await;
        let anthropic_mock = server.mock_async(|when, then| {
            when.method(POST)
                .path("/v1/complete");
            then.status(200)
                .json_body(json!({"completion": "anthropic reply"}));
        })
        .await;

        let mut registry = ProviderRegistry::new();
        registry.register_openai(
            "test-key".to_string(),
            Some("gpt-4o-mini".to_string()),
            Some(server.url("")),
        );
        registry.register_anthropic(
            "test-key".to_string(),
            Some("claude-3.0".to_string()),
            Some(server.url("")),
        );

        let openai_result = registry.think("OpenAI", "ping", &[], false).await.unwrap();
        let anthropic_result = registry.think("Anthropic", "ping", &[], false).await.unwrap();

        assert_eq!(openai_result, "openai reply");
        assert_eq!(anthropic_result, "anthropic reply");
        openai_mock.assert_async().await;
        anthropic_mock.assert_async().await;
    }

    #[tokio::test]
    async fn openai_provider_includes_history_in_request() {
        let server = MockServer::start_async().await;
        let mock = server.mock_async(|when, then| {
            when.method(POST)
                .path("/v1/chat/completions")
                .body_contains("assistant")
                .body_contains("past message");
            then.status(200)
                .json_body(json!({"choices": [{"message": {"content": "chat reply"}}]}));
        })
        .await;

        let provider = OpenAIProvider::new(
            "test-key".to_string(),
            None,
            Some(server.url("")),
        );

        let history = vec![ConversationMessage::new_assistant("past message".to_string(), false)];
        let response = provider.generate("follow up", &history).await.unwrap();
        assert_eq!(response, "chat reply");
        mock.assert_async().await;
    }
}
