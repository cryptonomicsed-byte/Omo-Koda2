use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

pub const SESSION_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Session {
    pub version: u32,
    pub messages: Vec<ConversationMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConversationMessage {
    pub role: MessageRole,
    pub blocks: Vec<ContentBlock>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContentBlock {
    Text {
        text: String,
    },
    ToolUse {
        id: String,
        name: String,
        input: String,
    },
    ToolResult {
        tool_use_id: String,
        tool_name: String,
        output: String,
        is_error: bool,
    },
}

impl Session {
    pub fn new() -> Self {
        Self {
            version: SESSION_VERSION,
            messages: Vec::new(),
        }
    }

    pub fn push_message(&mut self, message: ConversationMessage) {
        self.messages.push(message);
    }

    pub fn save_to_path(&self, path: impl AsRef<Path>) -> Result<(), SessionError> {
        let encoded = serde_json::to_string_pretty(self).map_err(SessionError::Encode)?;
        fs::write(path, encoded).map_err(SessionError::Io)
    }

    pub fn load_from_path(path: impl AsRef<Path>) -> Result<Self, SessionError> {
        let encoded = fs::read_to_string(path).map_err(SessionError::Io)?;
        let session: Session = serde_json::from_str(&encoded).map_err(SessionError::Decode)?;
        if session.version != SESSION_VERSION {
            return Err(SessionError::UnsupportedVersion(session.version));
        }
        Ok(session)
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

impl ConversationMessage {
    pub fn user_text(text: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            blocks: vec![ContentBlock::Text { text: text.into() }],
        }
    }

    pub fn assistant_text(text: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            blocks: vec![ContentBlock::Text { text: text.into() }],
        }
    }
}

#[derive(Debug)]
pub enum SessionError {
    Io(std::io::Error),
    Encode(serde_json::Error),
    Decode(serde_json::Error),
    UnsupportedVersion(u32),
}

impl std::fmt::Display for SessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionError::Io(error) => write!(f, "session I/O error: {error}"),
            SessionError::Encode(error) => write!(f, "session encode error: {error}"),
            SessionError::Decode(error) => write!(f, "session decode error: {error}"),
            SessionError::UnsupportedVersion(version) => {
                write!(f, "unsupported session version: {version}")
            }
        }
    }
}

impl std::error::Error for SessionError {}
