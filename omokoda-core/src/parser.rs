use std::collections::HashSet;

#[derive(Debug, PartialEq)]
pub struct MetadataPair {
    pub key: String,
    pub value: String,
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    Birth {
        name: String,
        metadata: Vec<MetadataPair>,
    },
    Think {
        prompt: String,
        private: bool,
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

#[derive(Debug)]
pub struct ParseError(pub String);

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ParseError: {}", self.0)
    }
}

const BLOCKED_IDENTIFIERS: &[&str] = &[
    "metabolism",
    "dopamine",
    "synapse",
    "ase",
    "àṣẹ",
    "odu_vault",
    "hermetic",
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
];

pub fn parse(input: &str) -> Result<Vec<Statement>, ParseError> {
    if input.len() > 4096 {
        return Err(ParseError("input exceeds 4096 bytes".into()));
    }

    let lower = input.to_lowercase();
    for blocked in BLOCKED_IDENTIFIERS {
        if lower.contains(blocked) {
            return Err(ParseError(format!(
                "internal identifier not allowed in input: {blocked}"
            )));
        }
    }

    if contains_raw_key_material(input) {
        return Err(ParseError("raw key material not allowed in input".into()));
    }

    let mut statements = Vec::new();
    for line in input.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        statements.push(parse_statement(line)?);
    }

    Ok(statements)
}

fn parse_statement(line: &str) -> Result<Statement, ParseError> {
    if line.starts_with('/') {
        return parse_slash_cmd(line);
    }

    let mut tokens = Tokenizer::new(line);
    match tokens.next_word().as_deref() {
        Some("birth") => parse_birth(&mut tokens),
        Some("think") => parse_think(&mut tokens),
        Some("act") => parse_act(&mut tokens),
        Some(_) => Ok(Statement::Think {
            prompt: line.to_string(),
            private: false,
        }),
        None => Err(ParseError("empty statement".into())),
    }
}

fn parse_birth(tokens: &mut Tokenizer) -> Result<Statement, ParseError> {
    let name = tokens
        .next_quoted_string()
        .ok_or_else(|| ParseError("birth requires a quoted name".into()))?;
    if name.is_empty() {
        return Err(ParseError("birth name cannot be empty".into()));
    }

    let mut metadata = Vec::new();
    while let Some(pair) = tokens.next_metadata_pair() {
        metadata.push(pair);
    }

    Ok(Statement::Birth { name, metadata })
}

fn parse_think(tokens: &mut Tokenizer) -> Result<Statement, ParseError> {
    let prompt = tokens
        .next_quoted_string()
        .ok_or_else(|| ParseError("think requires a quoted prompt".into()))?;
    if prompt.is_empty() {
        return Err(ParseError("think prompt cannot be empty".into()));
    }

    let private = tokens.next_flag("/private");
    Ok(Statement::Think { prompt, private })
}

fn parse_act(tokens: &mut Tokenizer) -> Result<Statement, ParseError> {
    let tool = tokens
        .next_quoted_string()
        .ok_or_else(|| ParseError("act requires a quoted tool name".into()))?;
    if tool.is_empty() {
        return Err(ParseError("act tool cannot be empty".into()));
    }

    let params = tokens
        .next_quoted_string()
        .ok_or_else(|| ParseError("act requires a quoted params string".into()))?;
    let sandbox = tokens.next_flag("/sandbox");
    Ok(Statement::Act {
        tool,
        params,
        sandbox,
    })
}

fn parse_slash_cmd(line: &str) -> Result<Statement, ParseError> {
    let rest = &line[1..];
    let mut parts = rest.splitn(2, ' ');
    let command = parts.next().unwrap_or("").trim().to_string();

    if !VALID_SLASH_COMMANDS.contains(&command.as_str()) {
        return Err(ParseError(format!("unknown slash command: /{command}")));
    }

    let arg = parts
        .next()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    Ok(Statement::SlashCmd { command, arg })
}

fn contains_raw_key_material(input: &str) -> bool {
    let hex_chars: HashSet<char> = "0123456789abcdefABCDEF".chars().collect();
    for word in input.split_whitespace() {
        let word = word.trim_matches('"');
        // Split on common separators to catch patterns like k_root:deadbeef1234
        for segment in word.split(|c| c == ':' || c == '=' || c == ',') {
            let mut s = segment;
            // Strip 0x prefix before checking
            if s.starts_with("0x") || s.starts_with("0X") {
                s = &s[2..];
            }
            // 8+ contiguous hex chars = raw key material
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
        while self.pos < self.input.len()
            && self.input.as_bytes()[self.pos].is_ascii_whitespace()
        {
            self.pos += 1;
        }
    }

    fn next_word(&mut self) -> Option<String> {
        self.skip_whitespace();
        let start = self.pos;
        while self.pos < self.input.len()
            && !self.input.as_bytes()[self.pos].is_ascii_whitespace()
        {
            self.pos += 1;
        }
        if self.pos == start {
            None
        } else {
            Some(self.input[start..self.pos].to_string())
        }
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

    fn next_flag(&mut self, flag: &str) -> bool {
        self.skip_whitespace();
        if self.input[self.pos..].starts_with(flag) {
            self.pos += flag.len();
            true
        } else {
            false
        }
    }
}
