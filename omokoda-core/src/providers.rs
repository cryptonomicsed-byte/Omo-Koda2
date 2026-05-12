use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderClass {
    Local,
    BrowserLocal,
    RegisteredLocal,
    External,
    Hive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMetadata {
    pub name: String,
    pub class: ProviderClass,
    pub endpoint: String,
}

pub struct ProviderRegistry {}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {}
    }

    pub fn is_allowed_in_private(&self, provider: &ProviderMetadata) -> bool {
        match provider.class {
            ProviderClass::Local => provider.endpoint.contains("localhost") || provider.endpoint.contains("127.0.0.1"),
            ProviderClass::BrowserLocal => true,
            ProviderClass::RegisteredLocal => true,
            ProviderClass::External => false,
            ProviderClass::Hive => false,
        }
    }

    pub fn validate_think(&self, provider: &ProviderMetadata, private_mode: bool) -> Result<(), String> {
        if private_mode && !self.is_allowed_in_private(provider) {
            return Err(format!(
                "privacy violation: provider '{}' ({:?}) is blocked in /private mode",
                provider.name, provider.class
            ));
        }
        Ok(())
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}
