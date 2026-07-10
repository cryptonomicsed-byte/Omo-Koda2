pub mod aliases;
pub mod streaming;

use crate::session::ConversationMessage;
use crate::tools::tool_definitions::{LlmResponse, ToolCall, ToolDefinition};
use crate::usage::TokenUsage;
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
    ) -> Result<(String, TokenUsage), String>;

    fn supports_tools(&self) -> bool {
        false
    }

    async fn generate_with_tools(
        &self,
        messages: &[ConversationMessage],
        _tools: &[ToolDefinition],
        _private: bool,
    ) -> Result<LlmResponse, String> {
        // Default: ignore tools, use last user message as prompt
        let prompt = messages
            .iter()
            .rev()
            .find(|m| m.role == crate::session::MessageRole::User)
            .map(|m| {
                m.blocks
                    .iter()
                    .filter_map(|b| match b {
                        crate::session::ContentBlock::Text { text } => Some(text.as_str()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .unwrap_or_default();
        let (text, usage) = self.generate(&prompt, messages).await?;
        Ok(LlmResponse::Text {
            content: text,
            usage,
        })
    }
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

        // OmniRoute — free OpenAI-compatible AI gateway (default localhost:8300),
        // no API key required. Registered ahead of Ollama so it is the working
        // default when no local model or BYOK key is present: this is how an
        // agent thinks "for free" out of the box. It routes to external models,
        // so it is class External (NOT eligible for /private thoughts, which
        // must stay on a local provider). Any BYOK provider (OpenAI/Anthropic/
        // LARQL) registered below lands ahead of it and takes priority.
        {
            let url = std::env::var("OMNIROUTE_URL")
                .ok()
                .filter(|u| !u.is_empty())
                .unwrap_or_else(|| "http://localhost:8300".to_string());
            // Default to a free, reliably-available direct model. The auto/*
            // combo router 503s under load ("Maximum combo retry limit"); a
            // pinned free model is steadier. Override with OMNIROUTE_MODEL.
            let model = std::env::var("OMNIROUTE_MODEL")
                .unwrap_or_else(|_| "oc/deepseek-v4-flash-free".to_string());
            // OmniRoute needs no key; send a non-empty dummy to avoid an empty
            // Authorization: Bearer header being rejected by some frontends.
            let token = std::env::var("OMNIROUTE_TOKEN")
                .unwrap_or_else(|_| "omniroute-free".to_string());
            registry.register(Box::new(OpenAIProvider::compatible(
                "omniroute",
                ProviderClass::External,
                token,
                model,
                url,
            )));
        }

        // LARQL — a local transformer decompiled into a queryable vindex,
        // served by larql-server's OpenAI-compatible surface. Weights stay on
        // this machine, so it qualifies as a Local (private-thought-eligible)
        // provider when LARQL_URL points at localhost.
        if let Ok(url) = std::env::var("LARQL_URL") {
            if !url.is_empty() {
                let token = std::env::var("LARQL_TOKEN").unwrap_or_default();
                let model = std::env::var("LARQL_MODEL").unwrap_or_else(|_| "default".to_string());
                registry.register(Box::new(OpenAIProvider::compatible(
                    "larql",
                    ProviderClass::Local,
                    token,
                    model,
                    url,
                )));
            }
        }

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

    pub async fn complete_with_tools(
        &self,
        provider_name: &str,
        messages: &[ConversationMessage],
        tools: &[ToolDefinition],
        private_mode: bool,
    ) -> Result<LlmResponse, String> {
        let provider = if provider_name.is_empty() || provider_name.eq_ignore_ascii_case("default")
        {
            self.providers.iter().map(Box::as_ref).find(|p| {
                if private_mode {
                    self.is_allowed_in_private(p.metadata())
                } else {
                    true
                }
            })
        } else {
            self.get_provider(provider_name)
        };

        let provider = provider.ok_or_else(|| "no provider available".to_string())?;

        if private_mode && !self.is_allowed_in_private(provider.metadata()) {
            return Err("No local provider available in /private mode (HARD FAIL)".to_string());
        }

        match tokio::time::timeout(
            Duration::from_secs(60),
            provider.generate_with_tools(messages, tools, private_mode),
        )
        .await
        {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(e)) => Err(format!("provider error: {}", e)),
            Err(_) => Err("provider timed out".to_string()),
        }
    }

    pub async fn think(
        &self,
        provider: &str,
        prompt: &str,
        history: &[ConversationMessage],
        private_mode: bool,
    ) -> Result<(String, TokenUsage), String> {
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
    ) -> Result<(String, TokenUsage), String> {
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
    ) -> Result<(String, TokenUsage), String> {
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
        let response = json["response"].as_str().unwrap_or("").to_string();

        let usage = TokenUsage {
            input_tokens: json["prompt_eval_count"].as_u64().unwrap_or(0) as u32,
            output_tokens: json["eval_count"].as_u64().unwrap_or(0) as u32,
            ..Default::default()
        };

        Ok((response, usage))
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
    ) -> Result<(String, TokenUsage), String> {
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

    /// Any server speaking the OpenAI chat-completions wire format, under its
    /// own provider name and class. This is how self-hosted engines (e.g. a
    /// local LARQL `larql-server`) join the registry without new wire code.
    pub fn compatible(
        name: &str,
        class: ProviderClass,
        api_key: String,
        model: String,
        endpoint: String,
    ) -> Self {
        Self {
            metadata: ProviderMetadata {
                name: name.to_string(),
                class,
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
    ) -> Result<(String, TokenUsage), String> {
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
            // Reasoning models (e.g. deepseek-v4-flash) spend tokens on hidden
            // reasoning before emitting the answer; too small a budget returns
            // empty content with finish_reason=length. Give real headroom.
            "max_tokens": 2000,
            // Must be explicit: OmniRoute (and some OpenAI-compatible gateways)
            // stream by default, returning text/event-stream chunks that would
            // break the single-object JSON parse below.
            "stream": false,
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
        let message = &json["choices"][0]["message"];
        // Prefer the answer; if a reasoning model left content empty, fall back
        // to its reasoning so a thought is never silently lost.
        let mut response = message["content"].as_str().unwrap_or("").trim().to_string();
        if response.is_empty() {
            response = message["reasoning_content"]
                .as_str()
                .unwrap_or("")
                .trim()
                .to_string();
        }
        if response.is_empty() {
            return Err("provider returned empty content".to_string());
        }

        let usage = TokenUsage {
            input_tokens: json["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as u32,
            output_tokens: json["usage"]["completion_tokens"].as_u64().unwrap_or(0) as u32,
            ..Default::default()
        };

        Ok((response, usage))
    }

    fn supports_tools(&self) -> bool {
        true
    }

    /// OpenAI-compatible function calling (works for DeepSeek, OmniRoute, and any
    /// OpenAI-compatible gateway). Maps the conversation — including prior tool
    /// calls and their results — into the chat/completions schema, advertises the
    /// tools as `type:function`, and parses `tool_calls` back out.
    async fn generate_with_tools(
        &self,
        messages: &[ConversationMessage],
        tools: &[ToolDefinition],
        _private: bool,
    ) -> Result<LlmResponse, String> {
        use crate::session::{ContentBlock, MessageRole};
        let url = if self.metadata.endpoint.contains("/v1/") {
            self.metadata.endpoint.clone()
        } else {
            format!(
                "{}/v1/chat/completions",
                self.metadata.endpoint.trim_end_matches('/')
            )
        };

        // Map conversation → OpenAI messages. Tool results become separate
        // role:"tool" messages (one per result); assistant tool calls carry a
        // tool_calls array.
        let mut api_messages: Vec<serde_json::Value> = Vec::new();
        for msg in messages {
            match msg.role {
                MessageRole::System | MessageRole::User => {
                    let text: String = msg
                        .blocks
                        .iter()
                        .filter_map(|b| match b {
                            ContentBlock::Text { text } => Some(text.clone()),
                            _ => None,
                        })
                        .collect::<Vec<_>>()
                        .join(" ");
                    let role = if msg.role == MessageRole::System {
                        "system"
                    } else {
                        "user"
                    };
                    if !text.is_empty() {
                        api_messages.push(serde_json::json!({"role": role, "content": text}));
                    }
                }
                MessageRole::Assistant => {
                    let mut text = String::new();
                    let mut tool_calls = Vec::new();
                    for b in &msg.blocks {
                        match b {
                            ContentBlock::Text { text: t } => text.push_str(t),
                            ContentBlock::ToolUse { id, name, input } => {
                                tool_calls.push(serde_json::json!({
                                    "id": id,
                                    "type": "function",
                                    "function": {"name": name, "arguments": input},
                                }));
                            }
                            _ => {}
                        }
                    }
                    let mut m = serde_json::json!({"role": "assistant"});
                    m["content"] = if text.is_empty() {
                        serde_json::Value::Null
                    } else {
                        serde_json::Value::String(text)
                    };
                    if !tool_calls.is_empty() {
                        m["tool_calls"] = serde_json::Value::Array(tool_calls);
                    }
                    api_messages.push(m);
                }
                MessageRole::Tool => {
                    // Each tool result is its own role:"tool" message.
                    for b in &msg.blocks {
                        if let ContentBlock::ToolResult {
                            tool_use_id,
                            output,
                            ..
                        } = b
                        {
                            api_messages.push(serde_json::json!({
                                "role": "tool",
                                "tool_call_id": tool_use_id,
                                "content": output,
                            }));
                        }
                    }
                }
            }
        }

        let api_tools: Vec<serde_json::Value> = tools
            .iter()
            .map(|t| {
                serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": t.name,
                        "description": t.description,
                        "parameters": {
                            "type": t.input_schema.type_,
                            "properties": t.input_schema.properties.iter().map(|(k, v)| {
                                let mut prop = serde_json::json!({"type": v.type_});
                                if let Some(desc) = &v.description {
                                    prop["description"] = serde_json::Value::String(desc.clone());
                                }
                                (k.clone(), prop)
                            }).collect::<std::collections::HashMap<_, _>>(),
                            "required": t.input_schema.required,
                        },
                    },
                })
            })
            .collect();

        let body = serde_json::json!({
            "model": self.model,
            "messages": api_messages,
            "tools": api_tools,
            "max_tokens": 2000,
            "stream": false,
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
            let status = resp.status();
            let err = resp.text().await.unwrap_or_default();
            return Err(format!("OpenAI status error {}: {}", status, err));
        }

        let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        let usage = TokenUsage {
            input_tokens: json["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as u32,
            output_tokens: json["usage"]["completion_tokens"].as_u64().unwrap_or(0) as u32,
            ..Default::default()
        };
        let message = &json["choices"][0]["message"];

        // Tool calls requested?
        if let Some(calls) = message["tool_calls"].as_array() {
            if !calls.is_empty() {
                let parsed: Vec<crate::tools::tool_definitions::ToolCall> = calls
                    .iter()
                    .enumerate()
                    .filter_map(|(i, c)| {
                        let f = &c["function"];
                        let name = f["name"].as_str()?.to_string();
                        let input = f["arguments"].as_str().unwrap_or("{}").to_string();
                        let id = c["id"]
                            .as_str()
                            .map(str::to_string)
                            .unwrap_or_else(|| format!("call_{i}"));
                        Some(crate::tools::tool_definitions::ToolCall { id, name, input })
                    })
                    .collect();
                let text_prefix = message["content"].as_str().map(str::to_string);
                return Ok(LlmResponse::ToolUse {
                    text_prefix,
                    calls: parsed,
                    usage,
                });
            }
        }

        // Plain text (with reasoning fallback, mirroring generate()).
        let mut content = message["content"].as_str().unwrap_or("").trim().to_string();
        if content.is_empty() {
            content = message["reasoning_content"]
                .as_str()
                .unwrap_or("")
                .trim()
                .to_string();
        }
        Ok(LlmResponse::Text { content, usage })
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

    async fn generate_with_tools_impl(
        &self,
        messages: &[ConversationMessage],
        tools: &[ToolDefinition],
        _private: bool,
    ) -> Result<LlmResponse, String> {
        let url = format!(
            "{}/v1/messages",
            self.metadata.endpoint.trim_end_matches('/')
        );

        // Build messages array for the Messages API
        let mut api_messages = Vec::new();
        for msg in messages {
            let role = match msg.role {
                crate::session::MessageRole::User => "user",
                crate::session::MessageRole::Assistant => "assistant",
                crate::session::MessageRole::Tool => "user", // tool results go as user role
                crate::session::MessageRole::System => continue,
            };

            let mut content_blocks = Vec::new();
            for block in &msg.blocks {
                match block {
                    crate::session::ContentBlock::Text { text } => {
                        content_blocks.push(serde_json::json!({
                            "type": "text",
                            "text": text
                        }));
                    }
                    crate::session::ContentBlock::ToolUse { id, name, input } => {
                        let input_value: serde_json::Value =
                            serde_json::from_str(input).unwrap_or(serde_json::json!({}));
                        content_blocks.push(serde_json::json!({
                            "type": "tool_use",
                            "id": id,
                            "name": name,
                            "input": input_value
                        }));
                    }
                    crate::session::ContentBlock::ToolResult {
                        tool_use_id,
                        output,
                        is_error,
                    } => {
                        content_blocks.push(serde_json::json!({
                            "type": "tool_result",
                            "tool_use_id": tool_use_id,
                            "content": output,
                            "is_error": is_error
                        }));
                    }
                }
            }

            if !content_blocks.is_empty() {
                api_messages.push(serde_json::json!({
                    "role": role,
                    "content": content_blocks
                }));
            }
        }

        // Build tools array
        let api_tools: Vec<serde_json::Value> = tools
            .iter()
            .map(|t| {
                serde_json::json!({
                    "name": t.name,
                    "description": t.description,
                    "input_schema": {
                        "type": t.input_schema.type_,
                        "properties": t.input_schema.properties.iter().map(|(k, v)| {
                            let mut prop = serde_json::json!({
                                "type": v.type_
                            });
                            if let Some(desc) = &v.description {
                                prop["description"] = serde_json::Value::String(desc.clone());
                            }
                            (k.clone(), prop)
                        }).collect::<std::collections::HashMap<_, _>>(),
                        "required": t.input_schema.required
                    }
                })
            })
            .collect();

        let body = serde_json::json!({
            "model": self.model,
            "max_tokens": 4096,
            "messages": api_messages,
            "tools": api_tools,
        });

        let resp = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            let status = resp.status();
            let err_body = resp.text().await.unwrap_or_default();
            return Err(format!("Anthropic status error {}: {}", status, err_body));
        }

        let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;

        let usage = TokenUsage {
            input_tokens: json["usage"]["input_tokens"].as_u64().unwrap_or(0) as u32,
            output_tokens: json["usage"]["output_tokens"].as_u64().unwrap_or(0) as u32,
            ..Default::default()
        };

        let stop_reason = json["stop_reason"].as_str().unwrap_or("");
        let content = json["content"].as_array().cloned().unwrap_or_default();

        // Collect text and tool_use blocks
        let mut text_parts = Vec::new();
        let mut tool_calls = Vec::new();

        for block in &content {
            match block["type"].as_str().unwrap_or("") {
                "text" => {
                    if let Some(t) = block["text"].as_str() {
                        text_parts.push(t.to_string());
                    }
                }
                "tool_use" => {
                    let id = block["id"].as_str().unwrap_or("").to_string();
                    let name = block["name"].as_str().unwrap_or("").to_string();
                    let input =
                        serde_json::to_string(&block["input"]).unwrap_or_else(|_| "{}".to_string());
                    tool_calls.push(ToolCall { id, name, input });
                }
                _ => {}
            }
        }

        if stop_reason == "tool_use" || !tool_calls.is_empty() {
            let text_prefix = if text_parts.is_empty() {
                None
            } else {
                Some(text_parts.join(""))
            };
            Ok(LlmResponse::ToolUse {
                text_prefix,
                calls: tool_calls,
                usage,
            })
        } else {
            Ok(LlmResponse::Text {
                content: text_parts.join(""),
                usage,
            })
        }
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    fn metadata(&self) -> &ProviderMetadata {
        &self.metadata
    }

    fn supports_tools(&self) -> bool {
        true
    }

    async fn generate_with_tools(
        &self,
        messages: &[ConversationMessage],
        tools: &[ToolDefinition],
        private: bool,
    ) -> Result<LlmResponse, String> {
        self.generate_with_tools_impl(messages, tools, private)
            .await
    }

    async fn generate(
        &self,
        prompt: &str,
        _history: &[ConversationMessage],
    ) -> Result<(String, TokenUsage), String> {
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
        let response = json["completion"].as_str().unwrap_or("").to_string();

        // Anthropic legacy /v1/complete might not return usage in the same way,
        // but we'll try to extract it if it's there.
        let usage = TokenUsage {
            input_tokens: json["usage"]["input_tokens"].as_u64().unwrap_or(0) as u32,
            output_tokens: json["usage"]["output_tokens"].as_u64().unwrap_or(0) as u32,
            ..Default::default()
        };

        Ok((response, usage))
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

    pub fn new_external(name: &str, response: String) -> Self {
        Self {
            metadata: ProviderMetadata {
                name: name.to_string(),
                class: ProviderClass::External,
                endpoint: "https://mock.api".to_string(),
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
        prompt: &str,
        _history: &[ConversationMessage],
    ) -> Result<(String, TokenUsage), String> {
        let usage = TokenUsage {
            input_tokens: prompt.split_whitespace().count() as u32,
            output_tokens: self.response.split_whitespace().count() as u32,
            ..Default::default()
        };
        Ok((self.response.clone(), usage))
    }

    fn supports_tools(&self) -> bool {
        true
    }

    async fn generate_with_tools(
        &self,
        _messages: &[ConversationMessage],
        _tools: &[ToolDefinition],
        _private: bool,
    ) -> Result<LlmResponse, String> {
        // Test helper: if response is "tool_call:name:input", emit a ToolUse response
        if let Some(rest) = self.response.strip_prefix("tool_call:") {
            let parts: Vec<&str> = rest.splitn(3, ':').collect();
            if parts.len() >= 2 {
                return Ok(LlmResponse::ToolUse {
                    text_prefix: None,
                    calls: vec![ToolCall {
                        id: "test-call-1".to_string(),
                        name: parts[0].to_string(),
                        input: if parts.len() > 2 {
                            parts[2].to_string()
                        } else {
                            "{}".to_string()
                        },
                    }],
                    usage: TokenUsage {
                        input_tokens: 10,
                        output_tokens: 5,
                        ..Default::default()
                    },
                });
            }
        }
        let usage = TokenUsage {
            input_tokens: 10,
            output_tokens: self.response.split_whitespace().count() as u32,
            ..Default::default()
        };
        Ok(LlmResponse::Text {
            content: self.response.clone(),
            usage,
        })
    }
}
