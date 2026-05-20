// Tool definition types for LLM-facing tool calling

use serde::{Deserialize, Serialize};
use crate::usage::TokenUsage;

/// JSON Schema property for tool input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolProperty {
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<String>>,
}

/// Input schema for a tool (JSON Schema subset)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInputSchema {
    #[serde(rename = "type")]
    pub type_: String,                           // always "object"
    pub properties: std::collections::HashMap<String, ToolProperty>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub required: Vec<String>,
}

/// Definition sent to the LLM so it knows what tools are available
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: ToolInputSchema,
}

/// A tool call requested by the LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub input: String,   // JSON string of params
}

/// LLM response that may contain text, tool calls, or both
#[derive(Debug, Clone)]
pub enum LlmResponse {
    Text {
        content: String,
        usage: TokenUsage,
    },
    ToolUse {
        text_prefix: Option<String>,   // text before tool calls (may be empty)
        calls: Vec<ToolCall>,
        usage: TokenUsage,
    },
}

impl LlmResponse {
    pub fn usage(&self) -> TokenUsage {
        match self {
            Self::Text { usage, .. } => *usage,
            Self::ToolUse { usage, .. } => *usage,
        }
    }

    pub fn has_tool_calls(&self) -> bool {
        matches!(self, Self::ToolUse { .. })
    }

    pub fn text_content(&self) -> Option<&str> {
        match self {
            Self::Text { content, .. } => Some(content.as_str()),
            Self::ToolUse { text_prefix, .. } => text_prefix.as_deref(),
        }
    }
}
