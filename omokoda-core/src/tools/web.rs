use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchInput {
    pub query: String,
    pub allowed_domains: Option<Vec<String>>,
    pub blocked_domains: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchOutput {
    pub results: Vec<WebSearchResultItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchResultItem {
    pub title: String,
    pub snippet: String,
    pub url: String,
}

pub fn web_search(input: WebSearchInput) -> Result<WebSearchOutput, String> {
    // Stub implementation
    Ok(WebSearchOutput { results: vec![] })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebFetchInput {
    pub url: String,
    pub prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebFetchOutput {
    pub content: String,
}

pub fn web_fetch(input: WebFetchInput) -> Result<WebFetchOutput, String> {
    // Stub implementation
    Ok(WebFetchOutput { content: String::new() })
}
