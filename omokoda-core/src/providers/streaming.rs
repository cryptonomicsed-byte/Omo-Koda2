use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TokenEvent {
    Token(String),
    ToolUseStart { id: String, name: String },
    ToolUseInput { id: String, input: String },
    ToolUseEnd { id: String },
    Done,
}

pub trait StreamingProvider {
    fn stream(&self, prompt: &str) -> Box<dyn Iterator<Item = TokenEvent>>;
}
