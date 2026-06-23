use std::collections::HashSet;

#[derive(Debug, PartialEq, Clone)]
pub struct MetadataPair {
    pub key: String,
    pub value: String,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct ThinkModifiers {
    pub loop_enabled: bool,
    pub max_iterations: Option<u32>,
    pub priority: Option<String>,
    pub sandbox: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Birth {
        name: String,
        metadata: Vec<MetadataPair>,
    },
    Think {
        prompt: String,
        private: bool,
        modifiers: ThinkModifiers,
    },
    Act {
        tool: String,
        params: String,
        sandbox: bool,
    },
    SlashCmd {
        command: String,
        arg: Option<String>,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseErrorCode {
    InvalidInput,
    BlockedIdentifier,
    RawKeyMaterial,
    MissingArgument,
    EmptyArgument,
    UnknownCommand,
    InputTooLong,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseError {
    pub code: ParseErrorCode,
    pub message: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:?}] {}", self.code, self.message)
    }
}

impl std::error::Error for ParseError {}

const MAX_INPUT: usize = 4096;

const BLOCKED_IDENTIFIERS: &[&str] = &[
    "odu_vault",
    "k_root",
    "k_0",
    "kdf",
    "walrus",
    "seal_vault",
    "bipọn",
    "ifascript",
    "soul.move",
    "agent.move",
    "hive.move",
];

const VALID_SLASH_COMMANDS: &[&str] = &[
    "private",
    "publish",
    "sandbox",
    "status",
    "transfer",
    "configure",
    "help",
    "tools",
    "unlock",
    "seal",
    "model",
    "export",
    "history",
    "skills",
    "receipts",
    "compact",
    "sessions",
    "memory",
    "clear",
    "think",
];

fn contains_blocked_identifiers(input: &str) -> bool {
    let lower_input = input.to_lowercase();
    for id in BLOCKED_IDENTIFIERS {
        for (pos, _) in lower_input.match_indices(id) {
            let before = if pos == 0 {
                None
            } else {
                lower_input.as_bytes().get(pos - 1).map(|&b| b as char)
            };
            let after = lower_input
                .as_bytes()
                .get(pos + id.len())
                .map(|&b| b as char);

            let before_is_word = before.is_some_and(|c| c.is_alphanumeric() || c == '_');
            let after_is_word = after.is_some_and(|c| c.is_alphanumeric() || c == '_');

            // If it's a standalone word or part of a technical identifier (with _), block it.
            // But we want to allow it if it's part of a DIFFERENT non-blocked word.
            // For these technical IDs, we usually want to block them even if they are prefixes.
            if !before_is_word && !after_is_word {
                return true;
            }
            // For technical ones like k_root, we might want to block even if followed by chars.
            if id.starts_with("k_") || id.contains('.') || id.contains('_') {
                return true;
            }
        }
    }
    false
}

pub fn parse(input: &str) -> Result<Vec<Statement>, ParseError> {
    if input.len() > MAX_INPUT {
        return Err(ParseError {
            code: ParseErrorCode::InputTooLong,
            message: format!("input exceeds max length of {} bytes", MAX_INPUT),
        });
    }

    if contains_blocked_identifiers(input) {
        return Err(ParseError {
            code: ParseErrorCode::BlockedIdentifier,
            message: "input contains blocked internal identifiers".into(),
        });
    }

    if contains_raw_key_material(input) {
        return Err(ParseError {
            code: ParseErrorCode::RawKeyMaterial,
            message: "input contains potential raw key material".into(),
        });
    }

    let mut statements = Vec::new();
    let mut tokens = Tokenizer::new(input);

    while !tokens.is_eof() {
        tokens.skip_whitespace();
        if tokens.is_eof() {
            break;
        }
        statements.push(parse_statement(&mut tokens)?);
    }
    Ok(statements)
}

fn parse_statement(tokens: &mut Tokenizer) -> Result<Statement, ParseError> {
    tokens.skip_whitespace();
    if tokens.peek_char() == Some('/') {
        return parse_slash_cmd(tokens);
    }

    match tokens.next_word().as_deref() {
        Some("birth") => parse_birth(tokens),
        Some("think") => parse_think(tokens),
        Some("act") => parse_act(tokens),
        Some(_) => Ok(Statement::Think {
            prompt: tokens.consume_rest_of_input_with_current_word(),
            private: true,
            modifiers: ThinkModifiers::default(),
        }),
        None => Err(ParseError {
            code: ParseErrorCode::EmptyArgument,
            message: "empty statement".into(),
        }),
    }
}

fn parse_slash_cmd(tokens: &mut Tokenizer) -> Result<Statement, ParseError> {
    tokens.pos += 1; // skip '/'
    let command = tokens.next_word().unwrap_or_default().trim().to_string();

    if !VALID_SLASH_COMMANDS.contains(&command.as_str()) {
        return Err(ParseError {
            code: ParseErrorCode::UnknownCommand,
            message: format!("unknown slash command: /{command}"),
        });
    }

    let arg = tokens.next_word().filter(|s| !s.is_empty());
    Ok(Statement::SlashCmd { command, arg })
}

fn parse_birth(tokens: &mut Tokenizer) -> Result<Statement, ParseError> {
    let name = tokens.next_quoted_string().ok_or_else(|| ParseError {
        code: ParseErrorCode::MissingArgument,
        message: "birth requires a quoted name".into(),
    })?;
    if name.is_empty() {
        return Err(ParseError {
            code: ParseErrorCode::EmptyArgument,
            message: "birth name cannot be empty".into(),
        });
    }

    let mut metadata = Vec::new();
    while let Some(pair) = tokens.next_metadata_pair() {
        metadata.push(pair);
    }

    Ok(Statement::Birth { name, metadata })
}

fn parse_think(tokens: &mut Tokenizer) -> Result<Statement, ParseError> {
    let prompt = tokens.next_quoted_string().ok_or_else(|| ParseError {
        code: ParseErrorCode::MissingArgument,
        message: "think requires a quoted prompt".into(),
    })?;
    if prompt.is_empty() {
        return Err(ParseError {
            code: ParseErrorCode::EmptyArgument,
            message: "think prompt cannot be empty".into(),
        });
    }

    let mut private = true;
    let mut modifiers = ThinkModifiers::default();
    while let Some(flag) = tokens.peek_word() {
        if flag == "/publish" {
            tokens.next_word();
            private = false;
        } else if flag == "/private" {
            tokens.next_word();
            private = true;
        } else if flag == "/sandbox" {
            tokens.next_word();
            modifiers.sandbox = true;
        } else if flag.contains(':') {
            let raw = tokens.next_word().unwrap_or_default();
            let (key, inline_value) = raw.split_once(':').unwrap_or((raw.as_str(), ""));
            let value = if inline_value.is_empty() {
                tokens.next_word().unwrap_or_default()
            } else {
                inline_value.to_string()
            };

            match key {
                "loop" => {
                    modifiers.loop_enabled = matches!(value.as_str(), "true" | "on" | "yes" | "1");
                }
                "max_iterations" => {
                    let parsed = value.parse::<u32>().map_err(|_| ParseError {
                        code: ParseErrorCode::InvalidInput,
                        message: "max_iterations must be a positive integer".into(),
                    })?;
                    modifiers.max_iterations = Some(parsed);
                }
                "priority" => {
                    if value.is_empty() {
                        return Err(ParseError {
                            code: ParseErrorCode::MissingArgument,
                            message: "priority requires a value".into(),
                        });
                    }
                    modifiers.priority = Some(value);
                }
                _ => {
                    return Err(ParseError {
                        code: ParseErrorCode::InvalidInput,
                        message: format!("unknown think modifier: {key}"),
                    });
                }
            }
        } else {
            break;
        }
    }
    Ok(Statement::Think {
        prompt,
        private,
        modifiers,
    })
}

fn parse_act(tokens: &mut Tokenizer) -> Result<Statement, ParseError> {
    let tool = tokens.next_quoted_string().ok_or_else(|| ParseError {
        code: ParseErrorCode::MissingArgument,
        message: "act requires a quoted tool name".into(),
    })?;
    if tool.is_empty() {
        return Err(ParseError {
            code: ParseErrorCode::EmptyArgument,
            message: "act tool cannot be empty".into(),
        });
    }

    let params = tokens.next_quoted_string().ok_or_else(|| ParseError {
        code: ParseErrorCode::MissingArgument,
        message: "act requires a quoted params string".into(),
    })?;

    let mut sandbox = false;
    while let Some(flag) = tokens.peek_word() {
        if flag == "/sandbox" {
            tokens.next_word();
            sandbox = true;
        } else {
            break;
        }
    }
    Ok(Statement::Act {
        tool,
        params,
        sandbox,
    })
}

fn contains_raw_key_material(input: &str) -> bool {
    let hex_chars: HashSet<char> = "0123456789abcdefABCDEF".chars().collect();
    for word in input.split_whitespace() {
        let word = word.trim_matches('"');
        for segment in word.split([':', '=', ',']) {
            let mut s = segment;
            if let Some(stripped) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
                s = stripped;
            }
            if s.len() >= 8 && s.chars().all(|c| hex_chars.contains(&c)) {
                return true;
            }
        }
    }
    false
}

struct Tokenizer<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Tokenizer<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() && self.input.as_bytes()[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn peek_char(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn peek_word(&mut self) -> Option<String> {
        let old_pos = self.pos;
        let word = self.next_word();
        self.pos = old_pos;
        word
    }

    fn next_word(&mut self) -> Option<String> {
        self.skip_whitespace();
        let start = self.pos;
        while self.pos < self.input.len() && !self.input.as_bytes()[self.pos].is_ascii_whitespace()
        {
            self.pos += 1;
        }
        if self.pos == start {
            None
        } else {
            Some(self.input[start..self.pos].to_string())
        }
    }

    fn consume_rest_of_input_with_current_word(&self) -> String {
        self.input[self.pos..].trim().to_string()
    }

    fn next_quoted_string(&mut self) -> Option<String> {
        self.skip_whitespace();
        let bytes = self.input.as_bytes();
        if self.pos >= bytes.len() || bytes[self.pos] != b'"' {
            return None;
        }
        self.pos += 1;
        let start = self.pos;
        while self.pos < bytes.len() && bytes[self.pos] != b'"' {
            self.pos += 1;
        }
        if self.pos >= bytes.len() {
            return None;
        }
        let s = self.input[start..self.pos].to_string();
        self.pos += 1;
        Some(s)
    }

    fn next_metadata_pair(&mut self) -> Option<MetadataPair> {
        self.skip_whitespace();
        let start = self.pos;
        while self.pos < self.input.len() {
            let b = self.input.as_bytes()[self.pos];
            if b == b':' {
                let key = self.input[start..self.pos].to_string();
                self.pos += 1;
                let val_start = self.pos;
                while self.pos < self.input.len()
                    && !self.input.as_bytes()[self.pos].is_ascii_whitespace()
                {
                    self.pos += 1;
                }
                let value = self.input[val_start..self.pos].to_string();
                if key.is_empty() || value.is_empty() {
                    return None;
                }
                return Some(MetadataPair { key, value });
            } else if b.is_ascii_whitespace() || b == b'"' {
                break;
            } else {
                self.pos += 1;
            }
        }
        self.pos = start;
        None
    }
}
