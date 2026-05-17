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
    async fn generate(
        &self,
        prompt: &str,
        history: &[ConversationMessage],
    ) -> Result<String, String>;
}

pub struct ProviderRegistry {
    pub providers: Vec<Box<dyn LlmProvider>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            providers: Vec::new(),
        };
        registry.register(Box::new(OllamaProvider::new(
            "http://localhost:11434".to_string(),
        )));
        registry.register(Box::new(WebLLMProvider::new()));

        if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
            registry.register(Box::new(OpenAIProvider::new(api_key, None, None)));
        }
        if let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY") {
            registry.register(Box::new(AnthropicProvider::new(api_key, None, None)));
        }

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
        self.providers.insert(0, provider);
    }

    pub fn register_openai(
        &mut self,
        api_key: String,
        model: Option<String>,
        endpoint: Option<String>,
    ) {
        self.register(Box::new(OpenAIProvider::new(api_key, model, endpoint)));
    }

    pub fn register_anthropic(
        &mut self,
        api_key: String,
        model: Option<String>,
        endpoint: Option<String>,
    ) {
        self.register(Box::new(AnthropicProvider::new(api_key, model, endpoint)));
    }

    pub fn is_allowed_in_private(&self, provider: &ProviderMetadata) -> bool {
        match provider.class {
            ProviderClass::Local => {
                provider.endpoint.contains("localhost")
                    || provider.endpoint.contains("127.0.0.1")
                    || provider.endpoint.contains("mock://")
            }
            ProviderClass::BrowserLocal => true,
            ProviderClass::RegisteredLocal => true,
            ProviderClass::External => false,
            ProviderClass::Hive => false,
        }
    }

    pub fn get_provider(&self, provider_name: &str) -> Option<&dyn LlmProvider> {
        let normalized = provider_name.to_lowercase();
        self.providers.iter().map(Box::as_ref).find(|provider| {
            let metadata = provider.metadata();
            metadata.name.to_lowercase() == normalized
                || metadata.endpoint.to_lowercase() == normalized
        })
    }

    pub fn provider_names(&self) -> Vec<String> {
        self.providers
            .iter()
            .map(|provider| provider.metadata().name.clone())
            .collect()
    }

    pub fn is_known_provider(&self, provider_name: &str) -> bool {
        self.get_provider(provider_name).is_some()
    }

    pub async fn think(
        &self,
        provider: &str,
        prompt: &str,
        history: &[ConversationMessage],
        private_mode: bool,
    ) -> Result<String, String> {
        let provider_name = provider.trim();
        if provider_name.is_empty() || provider_name.eq_ignore_ascii_case("default") {
            return self.route_think(prompt, history, private_mode).await;
        }

        let provider = self
            .get_provider(provider_name)
            .ok_or_else(|| format!("provider '{}' not found", provider_name))?;

        let metadata = provider.metadata();
        if private_mode && !self.is_allowed_in_private(metadata) {
            return Err("No local provider available in /private mode (HARD FAIL)".to_string());
        }

        match tokio::time::timeout(Duration::from_secs(30), provider.generate(prompt, history))
            .await
        {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(e)) => Err(format!("provider '{}' error: {}", provider_name, e)),
            Err(_) => Err(format!("provider '{}' timed out", provider_name)),
        }
    }

    fn provider_order(private_mode: bool) -> &'static [ProviderClass] {
        if private_mode {
            &[
                ProviderClass::Local,
                ProviderClass::BrowserLocal,
                ProviderClass::RegisteredLocal,
            ]
        } else {
            &[
                ProviderClass::Local,
                ProviderClass::BrowserLocal,
                ProviderClass::RegisteredLocal,
                ProviderClass::External,
                ProviderClass::Hive,
            ]
        }
    }

    pub async fn route_think(
        &self,
        prompt: &str,
        history: &[ConversationMessage],
        private_mode: bool,
    ) -> Result<String, String> {
        let order = Self::provider_order(private_mode);
        for provider_class in order {
            for provider in self
                .providers
                .iter()
                .filter(|p| p.metadata().class == *provider_class)
            {
                let metadata = provider.metadata();

                if private_mode && !self.is_allowed_in_private(metadata) {
                    continue;
                }

                match tokio::time::timeout(
                    Duration::from_secs(30),
                    provider.generate(prompt, history),
                )
                .await
                {
                    Ok(Ok(response)) => return Ok(response),
                    Ok(Err(_e)) => {
                        // Try next provider in the same class or next class
                    }
                    Err(_) => {
                        // Timeout, try next provider
                    }
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
            .field(
                "providers",
                &self
                    .providers
                    .iter()
                    .map(|p| p.metadata().name.clone())
                    .collect::<Vec<_>>(),
            )
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

    async fn generate(
        &self,
        prompt: &str,
        _history: &[ConversationMessage],
    ) -> Result<String, String> {
        let url = format!("{}/api/generate", self.metadata.endpoint);
        let body = serde_json::json!({
            "model": "llama3", // Default model
            "prompt": prompt,
            "stream": false
        });

        let resp = self
            .client
            .post(url)
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
pub struct WebLLMProvider {
    metadata: ProviderMetadata,
}

impl WebLLMProvider {
    pub fn new() -> Self {
        Self {
            metadata: ProviderMetadata {
                name: "WebLLM".to_string(),
                class: ProviderClass::BrowserLocal,
                endpoint: "browserllm://local".to_string(),
            },
        }
    }
}

impl Default for WebLLMProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LlmProvider for WebLLMProvider {
    fn metadata(&self) -> &ProviderMetadata {
        &self.metadata
    }

    async fn generate(
        &self,
        _prompt: &str,
        _history: &[ConversationMessage],
    ) -> Result<String, String> {
        Err("WebLLM provider not implemented".to_string())
    }
}

#[derive(Debug)]
pub struct OpenAIProvider {
    metadata: ProviderMetadata,
    client: reqwest::Client,
    api_key: String,
    model: String,
}

impl OpenAIProvider {
    pub fn new(api_key: String, model: Option<String>, endpoint: Option<String>) -> Self {
        let endpoint = endpoint.unwrap_or_else(|| "https://api.openai.com".to_string());
        let model = model.unwrap_or_else(|| "gpt-4o-mini".to_string());
        Self {
            metadata: ProviderMetadata {
                name: "OpenAI".to_string(),
                class: ProviderClass::External,
                endpoint,
            },
            client: reqwest::Client::new(),
            api_key,
            model,
        }
    }
}

#[async_trait]
impl LlmProvider for OpenAIProvider {
    fn metadata(&self) -> &ProviderMetadata {
        &self.metadata
    }

    async fn generate(
        &self,
        prompt: &str,
        history: &[ConversationMessage],
    ) -> Result<String, String> {
        let url = if self.metadata.endpoint.contains("/v1/") {
            self.metadata.endpoint.clone()
        } else {
            format!(
                "{}/v1/chat/completions",
                self.metadata.endpoint.trim_end_matches('/')
            )
        };

        let mut messages = Vec::new();
        for message in history {
            let role = match message.role {
                crate::session::MessageRole::User => "user",
                crate::session::MessageRole::Assistant => "assistant",
                crate::session::MessageRole::System => "system",
                _ => "user",
            };
            let content = message
                .blocks
                .iter()
                .map(|block| match block {
                    crate::session::ContentBlock::Text { text } => text.clone(),
                    crate::session::ContentBlock::ToolResult { output, .. } => output.clone(),
                    crate::session::ContentBlock::ToolUse { input, .. } => input.clone(),
                })
                .collect::<Vec<_>>()
                .join(" ");
            if !content.is_empty() {
                messages.push(serde_json::json!({"role": role, "content": content}));
            }
        }
        messages.push(serde_json::json!({"role": "user", "content": prompt}));

        let body = serde_json::json!({
            "model": self.model,
            "messages": messages,
            "temperature": 0.7,
            "max_tokens": 500,
        });

        let resp = self
            .client
            .post(url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            return Err(format!("OpenAI status error: {}", resp.status()));
        }

        let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        Ok(json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string())
    }
}

#[derive(Debug)]
pub struct AnthropicProvider {
    metadata: ProviderMetadata,
    client: reqwest::Client,
    api_key: String,
    model: String,
}

impl AnthropicProvider {
    pub fn new(api_key: String, model: Option<String>, endpoint: Option<String>) -> Self {
        let endpoint = endpoint.unwrap_or_else(|| "https://api.anthropic.com".to_string());
        let model = model.unwrap_or_else(|| "claude-3.0".to_string());
        Self {
            metadata: ProviderMetadata {
                name: "Anthropic".to_string(),
                class: ProviderClass::External,
                endpoint,
            },
            client: reqwest::Client::new(),
            api_key,
            model,
        }
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    fn metadata(&self) -> &ProviderMetadata {
        &self.metadata
    }

    async fn generate(
        &self,
        prompt: &str,
        _history: &[ConversationMessage],
    ) -> Result<String, String> {
        let url = if self.metadata.endpoint.contains("/v1/") {
            self.metadata.endpoint.clone()
        } else {
            format!(
                "{}/v1/complete",
                self.metadata.endpoint.trim_end_matches('/')
            )
        };

        let body = serde_json::json!({
            "model": self.model,
            "prompt": format!("\n\nHuman: {}\n\nAssistant:", prompt),
            "max_tokens_to_sample": 500,
            "temperature": 0.7,
        });

        let resp = self
            .client
            .post(url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            return Err(format!("Anthropic status error: {}", resp.status()));
        }

        let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        Ok(json["completion"].as_str().unwrap_or("").to_string())
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
                name: "ollama".to_string(),
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

    async fn generate(
        &self,
        _prompt: &str,
        _history: &[ConversationMessage],
    ) -> Result<String, String> {
        Ok(self.response.clone())
    }
}
