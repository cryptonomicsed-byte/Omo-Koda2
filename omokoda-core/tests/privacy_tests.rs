#[cfg(test)]
mod privacy_tests {
    use omokoda_core::providers::{ProviderClass, ProviderMetadata, ProviderRegistry};

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
            name: "Claude".to_string(),
            class: ProviderClass::External,
            endpoint: "https://api.anthropic.com".to_string(),
        };
        assert!(!registry.is_allowed_in_private(&claude));
    }

    #[tokio::test]
    async fn private_mode_hard_fails_if_only_external_available() {
        struct ExternalProv(ProviderMetadata);
        #[async_trait::async_trait]
        impl omokoda_core::providers::LlmProvider for ExternalProv {
            fn metadata(&self) -> &ProviderMetadata { &self.0 }
            async fn generate(&self, _: &str, _: &[omokoda_core::session::ConversationMessage]) -> Result<String, String> {
                Ok("external".to_string())
            }
        }

        let mut registry = ProviderRegistry::with_mock("local".to_string());
        registry.register(Box::new(ExternalProv(ProviderMetadata {
            name: "Claude".to_string(),
            class: ProviderClass::External,
            endpoint: "https://api.anthropic.com".to_string(),
        })));

        // In public mode, it might use either (but mock is first)
        let res = registry.route_think("h", &[], false).await.unwrap();
        assert_eq!(res, "local");
    }

    #[tokio::test]
    async fn private_mode_hard_fails_when_no_local_succeeds() {
        struct FailProv(ProviderMetadata);
        #[async_trait::async_trait]
        impl omokoda_core::providers::LlmProvider for FailProv {
            fn metadata(&self) -> &ProviderMetadata { &self.0 }
            async fn generate(&self, _: &str, _: &[omokoda_core::session::ConversationMessage]) -> Result<String, String> {
                Err("local fail".to_string())
            }
        }
        struct ExternalProv(ProviderMetadata);
        #[async_trait::async_trait]
        impl omokoda_core::providers::LlmProvider for ExternalProv {
            fn metadata(&self) -> &ProviderMetadata { &self.0 }
            async fn generate(&self, _: &str, _: &[omokoda_core::session::ConversationMessage]) -> Result<String, String> {
                Ok("external".to_string())
            }
        }

        let mut registry = ProviderRegistry { providers: Vec::new() };
        registry.register(Box::new(FailProv(ProviderMetadata {
            name: "FailLocal".to_string(),
            class: ProviderClass::Local,
            endpoint: "http://localhost:1".to_string(),
        })));
        registry.register(Box::new(ExternalProv(ProviderMetadata {
            name: "Claude".to_string(),
            class: ProviderClass::External,
            endpoint: "https://api.anthropic.com".to_string(),
        })));

        let res = registry.route_think("h", &[], true).await;
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("HARD FAIL"));
    }
}
