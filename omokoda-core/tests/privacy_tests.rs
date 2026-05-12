#[cfg(test)]
mod privacy_tests {
    use omokoda_core::providers::{ProviderClass, ProviderMetadata, ProviderRegistry};

    #[test]
    fn allows_local_in_private() {
        let registry = ProviderRegistry::new();
        let ollama = ProviderMetadata {
            name: "Ollama".to_string(),
            class: ProviderClass::Local,
            endpoint: "http://localhost:11434".to_string(),
        };
        assert!(registry.validate_think(&ollama, true).is_ok());
    }

    #[test]
    fn blocks_external_in_private() {
        let registry = ProviderRegistry::new();
        let claude = ProviderMetadata {
            name: "Claude".to_string(),
            class: ProviderClass::External,
            endpoint: "https://api.anthropic.com".to_string(),
        };
        let result = registry.validate_think(&claude, true);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("privacy violation"));
    }

    #[test]
    fn allows_external_in_public() {
        let registry = ProviderRegistry::new();
        let claude = ProviderMetadata {
            name: "Claude".to_string(),
            class: ProviderClass::External,
            endpoint: "https://api.anthropic.com".to_string(),
        };
        assert!(registry.validate_think(&claude, false).is_ok());
    }

    #[test]
    fn blocks_hive_in_private() {
        let registry = ProviderRegistry::new();
        let hive = ProviderMetadata {
            name: "Hive Node 1".to_string(),
            class: ProviderClass::Hive,
            endpoint: "https://hive.omokoda.io".to_string(),
        };
        assert!(registry.validate_think(&hive, true).is_err());
    }
}
