use reqwest::Client;
use serde_json::Value;
use std::sync::OnceLock;

static HTTP: OnceLock<Client> = OnceLock::new();
fn http() -> &'static Client {
    HTTP.get_or_init(Client::new)
}

pub struct WorkspaceClient {
    pub base_url: String,
    pub api_key: String,
    pub guild_slug: String,
}

impl WorkspaceClient {
    /// Load from environment variables: VANTAGE_URL, VANTAGE_KEY, VANTAGE_GUILD_SLUG
    /// Returns None if VANTAGE_URL is not set.
    pub fn from_env() -> Option<Self> {
        let base = std::env::var("VANTAGE_URL").ok()?;
        if base.trim().is_empty() { return None; }
        let api_key = std::env::var("VANTAGE_KEY").unwrap_or_default();
        let guild_slug = std::env::var("VANTAGE_GUILD_SLUG").unwrap_or_default();
        Some(Self {
            base_url: base.trim_end_matches('/').to_string(),
            api_key,
            guild_slug,
        })
    }

    pub fn guild_path(&self, suffix: &str) -> String {
        format!("{}/api/guilds/{}{}", self.base_url, self.guild_slug, suffix)
    }

    pub(crate) fn task_base(&self) -> String {
        self.guild_path("/tasks")
    }

    pub(crate) fn memory_base(&self) -> String {
        self.guild_path("/memory")
    }

    pub async fn get(&self, url: &str) -> Result<Value, String> {
        let resp = http()
            .get(url)
            .header("X-Agent-Key", &self.api_key)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let status = resp.status();
        let val: Value = resp.json().await.unwrap_or(Value::Null);
        if !status.is_success() {
            return Err(format!("GET {url} -> {status}: {val}"));
        }
        Ok(val)
    }

    pub async fn post_form(&self, url: &str, form: &[(&str, &str)]) -> Result<Value, String> {
        let resp = http()
            .post(url)
            .header("X-Agent-Key", &self.api_key)
            .form(form)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let status = resp.status();
        let val: Value = resp.json().await.unwrap_or(Value::Null);
        if !status.is_success() {
            return Err(format!("POST {url} -> {status}: {val}"));
        }
        Ok(val)
    }

    pub async fn delete(&self, url: &str) -> Result<Value, String> {
        let resp = http()
            .delete(url)
            .header("X-Agent-Key", &self.api_key)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let status = resp.status();
        let val: Value = resp.json().await.unwrap_or(Value::Null);
        if !status.is_success() {
            return Err(format!("DELETE {url} -> {status}: {val}"));
        }
        Ok(val)
    }

    pub async fn post_json(&self, url: &str, body: Value) -> Result<Value, String> {
        let resp = http()
            .post(url)
            .header("X-Agent-Key", &self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let status = resp.status();
        let val: Value = resp.json().await.unwrap_or(Value::Null);
        if !status.is_success() {
            return Err(format!("POST {url} -> {status}: {val}"));
        }
        Ok(val)
    }

    pub async fn put_form(&self, url: &str, form: &[(&str, &str)]) -> Result<Value, String> {
        let resp = http()
            .put(url)
            .header("X-Agent-Key", &self.api_key)
            .form(form)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let status = resp.status();
        let val: Value = resp.json().await.unwrap_or(Value::Null);
        if !status.is_success() {
            return Err(format!("PUT {url} -> {status}: {val}"));
        }
        Ok(val)
    }
}
