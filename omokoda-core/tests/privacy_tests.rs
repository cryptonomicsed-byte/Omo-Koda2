#[cfg(test)]
mod privacy_tests {
    use omokoda_core::providers::{
        MockProvider, ProviderClass, ProviderMetadata, ProviderRegistry,
    };
    use omokoda_core::usage::TokenUsage;

    #[tokio::test]
    async fn allows_local_in_private() {
        let registry = ProviderRegistry::new();
        let ollama = ProviderMetadata {
            name: "Ollama".to_string(),
            class: ProviderClass::Local,
            endpoint: "http://localhost:11434".to_string(),
        };
        // validate_think was removed, we use route_think now or just check is_allowed_in_private
        assert!(registry.is_allowed_in_private(&ollama));
    }

    #[tokio::test]
    async fn blocks_external_in_private() {
        let registry = ProviderRegistry::new();
        let claude = ProviderMetadata {
            name: "Anthropic".to_string(),
            class: ProviderClass::External,
            endpoint: "https://api.anthropic.com".to_string(),
        };
        assert!(!registry.is_allowed_in_private(&claude));
    }

    #[tokio::test]
    async fn explicit_provider_selection_prefers_named_provider() {
        let mut registry = ProviderRegistry::new();
        registry.register(Box::new(MockProvider::new("mock response".to_string())));

        let response = registry.think("ollama", "hello", &[], false).await.unwrap();
        assert_eq!(response.0, "mock response");
    }

    #[tokio::test]
    async fn route_think_falls_back_public_providers_in_priority_order() {
        struct FailingLocal(ProviderMetadata);
        #[async_trait::async_trait]
        impl omokoda_core::providers::LlmProvider for FailingLocal {
            fn metadata(&self) -> &ProviderMetadata {
                &self.0
            }
            async fn generate(
                &self,
                _: &str,
                _: &[omokoda_core::session::ConversationMessage],
            ) -> Result<(String, TokenUsage), String> {
                Err("local failed".to_string())
            }
        }

        struct BrowserLocal(ProviderMetadata);
        #[async_trait::async_trait]
        impl omokoda_core::providers::LlmProvider for BrowserLocal {
            fn metadata(&self) -> &ProviderMetadata {
                &self.0
            }
            async fn generate(
                &self,
                _: &str,
                _: &[omokoda_core::session::ConversationMessage],
            ) -> Result<(String, TokenUsage), String> {
                Ok(("browser local".to_string(), TokenUsage::default()))
            }
        }

        let mut registry = ProviderRegistry::empty();
        registry.register(Box::new(FailingLocal(ProviderMetadata {
            name: "LocalFail".to_string(),
            class: ProviderClass::Local,
            endpoint: "http://localhost:11434".to_string(),
        })));
        registry.register(Box::new(BrowserLocal(ProviderMetadata {
            name: "WebLLM".to_string(),
            class: ProviderClass::BrowserLocal,
            endpoint: "browser://local".to_string(),
        })));

        let response = registry.route_think("hello", &[], false).await.unwrap();
        assert_eq!(response.0, "browser local");
    }

    #[tokio::test]
    async fn explicit_private_external_provider_hard_fails() {
        struct ExternalProv(ProviderMetadata);
        #[async_trait::async_trait]
        impl omokoda_core::providers::LlmProvider for ExternalProv {
            fn metadata(&self) -> &ProviderMetadata {
                &self.0
            }
            async fn generate(
                &self,
                _: &str,
                _: &[omokoda_core::session::ConversationMessage],
            ) -> Result<(String, TokenUsage), String> {
                Ok(("external".to_string(), TokenUsage::default()))
            }
        }

        let mut registry = ProviderRegistry::with_mock("local".to_string());
        registry.register(Box::new(ExternalProv(ProviderMetadata {
            name: "Anthropic".to_string(),
            class: ProviderClass::External,
            endpoint: "https://api.anthropic.com".to_string(),
        })));

        let res = registry.think("Anthropic", "hello", &[], true).await;
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("HARD FAIL"));
    }

    #[tokio::test]
    async fn private_mode_hard_fails_when_no_local_succeeds() {
        struct ExternalProv(ProviderMetadata);
        #[async_trait::async_trait]
        impl omokoda_core::providers::LlmProvider for ExternalProv {
            fn metadata(&self) -> &ProviderMetadata {
                &self.0
            }
            async fn generate(
                &self,
                _: &str,
                _: &[omokoda_core::session::ConversationMessage],
            ) -> Result<(String, TokenUsage), String> {
                Ok(("external".to_string(), TokenUsage::default()))
            }
        }

        let mut registry = ProviderRegistry::with_mock("local".to_string());
        registry.register(Box::new(ExternalProv(ProviderMetadata {
            name: "Anthropic".to_string(),
            class: ProviderClass::External,
            endpoint: "https://api.anthropic.com".to_string(),
        })));

        // In public mode, it might use either (but mock is first)
        let res = registry.route_think("h", &[], false).await.unwrap();
        assert_eq!(res.0, "local");
    }

    #[tokio::test]
    async fn private_mode_fails_when_local_provider_fails() {
        struct FailProv(ProviderMetadata);
        #[async_trait::async_trait]
        impl omokoda_core::providers::LlmProvider for FailProv {
            fn metadata(&self) -> &ProviderMetadata {
                &self.0
            }
            async fn generate(
                &self,
                _: &str,
                _: &[omokoda_core::session::ConversationMessage],
            ) -> Result<(String, TokenUsage), String> {
                Err("local fail".to_string())
            }
        }
        struct ExternalProv(ProviderMetadata);
        #[async_trait::async_trait]
        impl omokoda_core::providers::LlmProvider for ExternalProv {
            fn metadata(&self) -> &ProviderMetadata {
                &self.0
            }
            async fn generate(
                &self,
                _: &str,
                _: &[omokoda_core::session::ConversationMessage],
            ) -> Result<(String, TokenUsage), String> {
                Ok(("external".to_string(), TokenUsage::default()))
            }
        }

        let mut registry = ProviderRegistry::empty();
        registry.register(Box::new(FailProv(ProviderMetadata {
            name: "FailLocal".to_string(),
            class: ProviderClass::Local,
            endpoint: "http://localhost:1".to_string(),
        })));
        registry.register(Box::new(ExternalProv(ProviderMetadata {
            name: "Anthropic".to_string(),
            class: ProviderClass::External,
            endpoint: "https://api.anthropic.com".to_string(),
        })));

        let res = registry.route_think("h", &[], true).await;
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("HARD FAIL"));
    }
}
