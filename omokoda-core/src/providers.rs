use crate::session::ConversationMessage;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

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

#[async_trait]
pub trait LlmProvider: Send + Sync {
    fn metadata(&self) -> &ProviderMetadata;
    async fn generate(&self, prompt: &str, history: &[ConversationMessage]) -> Result<String, String>;
}

pub struct ProviderRegistry {
    pub providers: Vec<Box<dyn LlmProvider>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            providers: Vec::new(),
        };
        registry.register(Box::new(OllamaProvider::new("http://localhost:11434".to_string())));
        registry
    }

    pub fn with_mock(response: String) -> Self {
        let mut registry = Self {
            providers: Vec::new(),
        };
        registry.register(Box::new(MockProvider::new(response)));
        registry
    }

    pub fn register(&mut self, provider: Box<dyn LlmProvider>) {
        self.providers.push(provider);
    }

    pub fn is_allowed_in_private(&self, provider: &ProviderMetadata) -> bool {
        match provider.class {
            ProviderClass::Local => {
                provider.endpoint.contains("localhost") || provider.endpoint.contains("127.0.0.1") || provider.endpoint.contains("mock://")
            }
            ProviderClass::BrowserLocal => true,
            ProviderClass::RegisteredLocal => true,
            ProviderClass::External => false,
            ProviderClass::Hive => false,
        }
    }

    pub async fn think(&self, _provider: &str, prompt: &str) -> Result<String, String> {
        // Simplified for now: route based on default logic
        // In the future, this can use the specific provider name
        self.route_think(prompt, &[], false).await
    }

    pub async fn route_think(

        &self,
        prompt: &str,
        history: &[ConversationMessage],
        private_mode: bool,
    ) -> Result<String, String> {
        for provider in &self.providers {
            let metadata = provider.metadata();
            
            if private_mode && !self.is_allowed_in_private(metadata) {
                continue;
            }

            match tokio::time::timeout(Duration::from_secs(30), provider.generate(prompt, history)).await {
                Ok(Ok(response)) => return Ok(response),
                Ok(Err(_e)) => {
                    // Try next provider
                }
                Err(_) => {
                    // Timeout, try next provider
                }
            }
        }

        if private_mode {
            Err("No local provider available in /private mode (HARD FAIL)".to_string())
        } else {
            Err("Reasoning failed: no provider responded".to_string())
        }
    }
}

impl std::fmt::Debug for ProviderRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProviderRegistry")
            .field("providers", &self.providers.iter().map(|p| p.metadata().name.clone()).collect::<Vec<_>>())
            .finish()
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct OllamaProvider {
    metadata: ProviderMetadata,
    client: reqwest::Client,
}

impl OllamaProvider {
    pub fn new(endpoint: String) -> Self {
        Self {
            metadata: ProviderMetadata {
                name: "Ollama".to_string(),
                class: ProviderClass::Local,
                endpoint,
            },
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    fn metadata(&self) -> &ProviderMetadata {
        &self.metadata
    }

    async fn generate(&self, prompt: &str, _history: &[ConversationMessage]) -> Result<String, String> {
        let url = format!("{}/api/generate", self.metadata.endpoint);
        let body = serde_json::json!({
            "model": "llama3", // Default model
            "prompt": prompt,
            "stream": false
        });

        let resp = self.client.post(url)
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            return Err(format!("Ollama status error: {}", resp.status()));
        }

        let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        Ok(json["response"].as_str().unwrap_or("").to_string())
    }
}

#[derive(Debug)]
pub struct MockProvider {
    pub metadata: ProviderMetadata,
    pub response: String,
}

impl MockProvider {
    pub fn new(response: String) -> Self {
        Self {
            metadata: ProviderMetadata {
                name: "Mock".to_string(),
                class: ProviderClass::Local,
                endpoint: "mock://".to_string(),
            },
            response,
        }
    }
}

#[async_trait]
impl LlmProvider for MockProvider {
    fn metadata(&self) -> &ProviderMetadata {
        &self.metadata
    }

    async fn generate(&self, _prompt: &str, _history: &[ConversationMessage]) -> Result<String, String> {
        Ok(self.response.clone())
    }
}
