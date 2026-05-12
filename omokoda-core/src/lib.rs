pub mod identity;
pub mod interpreter;
pub mod parser;
pub mod receipt;
pub mod reputation;
pub mod session;

use chrono::{DateTime, Utc};
use rand::Rng;
use reputation::{reputation_gain, tier_for, tools_for_tier};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub dna: Vec<u8>,
    pub personality: PersonalityVector,
    pub consistency_score: f32,
    pub last_activity: DateTime<Utc>,
    pub decay_rate: f32,
    pub polarity_balance: f32,
    pub action_cooldowns: HashMap<String, DateTime<Utc>>,
    pub receipt_chain: Vec<Receipt>,
    pub think_count: u32,
    pub act_count: u32,
    pub gender_balance: f32,
    pub reputation: f64,
    pub tier: u8,
    pub memory_key: Vec<u8>,
    pub odu_state: OduState,
    pub working_memory: Vec<MemoryEntry>,
    pub short_term_memory: Vec<MemoryEntry>,
    pub long_term_memory: Vec<MemoryEntry>,
    pub unlocked_tools: Vec<String>,
    pub sandbox_mode: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PersonalityVector {
    pub curiosity: f32,
    pub boldness: f32,
    pub empathy: f32,
    pub wisdom: f32,
    pub creativity: f32,
}

impl PersonalityVector {
    pub fn neutral() -> Self {
        Self {
            curiosity: 0.5,
            boldness: 0.5,
            empathy: 0.5,
            wisdom: 0.5,
            creativity: 0.5,
        }
    }

    pub fn evolve(&mut self, delta: PersonalityVector) {
        self.curiosity = (self.curiosity + delta.curiosity).clamp(0.0, 1.0);
        self.boldness = (self.boldness + delta.boldness).clamp(0.0, 1.0);
        self.empathy = (self.empathy + delta.empathy).clamp(0.0, 1.0);
        self.wisdom = (self.wisdom + delta.wisdom).clamp(0.0, 1.0);
        self.creativity = (self.creativity + delta.creativity).clamp(0.0, 1.0);
    }

    pub fn boost_curiosity(&mut self, amt: f32) {
        self.curiosity = (self.curiosity + amt).min(1.0);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OduState {
    pub index: u32,
    pub seed: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Receipt {
    pub id: String,
    pub action: String,
    pub timestamp: DateTime<Utc>,
    pub summary: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MemoryEntry {
    pub id: String,
    pub scope: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub enum Primitive {
    Birth {
        name: String,
        metadata: Vec<(String, String)>,
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
}

#[derive(Debug, Clone)]
pub struct AgentRuntime {
    pub agent: Agent,
}

impl Agent {
    pub fn hidden_state_summary(&self) -> String {
        format!(
            "agent={} tier={} rep={:.3} think={} act={} sandbox={}",
            self.name,
            self.tier,
            self.reputation,
            self.think_count,
            self.act_count,
            self.sandbox_mode
        )
    }

    pub fn create(name: String) -> Self {
        let now = Utc::now();
        let mut rng = rand::thread_rng();
        let mut dna = vec![0u8; 32];
        rng.fill(&mut dna[..]);

        Self {
            id: format!("agent-{}", hex_hash(&name)),
            name,
            dna,
            personality: PersonalityVector::neutral(),
            consistency_score: 0.5,
            last_activity: now,
            decay_rate: 0.01,
            polarity_balance: 0.5,
            action_cooldowns: HashMap::new(),
            receipt_chain: vec![],
            think_count: 0,
            act_count: 0,
            gender_balance: 0.5,
            reputation: 0.0,
            tier: 0,
            memory_key: vec![],
            odu_state: OduState {
                index: 0,
                seed: vec![],
            },
            working_memory: vec![],
            short_term_memory: vec![],
            long_term_memory: vec![],
            unlocked_tools: vec!["web_search".to_string()],
            sandbox_mode: false,
        }
    }
}

impl AgentRuntime {
    pub fn birth(name: String) -> Self {
        Self {
            agent: Agent::create(name),
        }
    }

    pub fn think(&mut self, prompt: &str, private: bool) -> String {
        self.agent.think_count += 1;
        self.agent.last_activity = Utc::now();

        let scope = if private { "private" } else { "public" };
        self.agent.working_memory.push(MemoryEntry {
            id: format!("mem-{}", self.agent.think_count),
            scope: scope.to_string(),
            content: prompt.to_string(),
            created_at: Utc::now(),
        });

        if private {
            self.agent.short_term_memory.push(MemoryEntry {
                id: format!("priv-{}", self.agent.think_count),
                scope: scope.to_string(),
                content: prompt.to_string(),
                created_at: Utc::now(),
            });
        }

        format!("thought:{}", prompt)
    }

    pub fn act(&mut self, tool: &str, params: &str, sandbox: bool) -> Receipt {
        self.agent.act_count += 1;
        self.agent.last_activity = Utc::now();
        self.agent.sandbox_mode = sandbox;

        let summary = format!("tool={} params={} sandbox={}", tool, params, sandbox);
        let receipt = Receipt {
            id: format!("rcpt-{}", self.agent.act_count),
            action: tool.to_string(),
            timestamp: Utc::now(),
            summary,
        };
        self.agent.receipt_chain.push(receipt.clone());
        self.agent.reputation =
            (self.agent.reputation + reputation_gain(0.040, self.agent.reputation)).min(100.0);
        self.agent.tier = tier_for(self.agent.reputation);
        self.agent.unlocked_tools = tools_for_tier(self.agent.tier);
        receipt
    }
}

pub fn parse_primitive(input: &str) -> Option<Primitive> {
    let trimmed = input.trim();
    if let Some(rest) = trimmed.strip_prefix("birth ") {
        let name = rest.trim().trim_matches('"').to_string();
        return Some(Primitive::Birth {
            name,
            metadata: vec![],
        });
    }
    if let Some(rest) = trimmed.strip_prefix("think ") {
        let prompt = rest.trim().trim_matches('"').to_string();
        return Some(Primitive::Think {
            prompt,
            private: true,
        });
    }
    if let Some(rest) = trimmed.strip_prefix("act ") {
        let mut parts = rest.splitn(2, ' ');
        let tool = parts
            .next()
            .unwrap_or("")
            .trim()
            .trim_matches('"')
            .to_string();
        let params = parts
            .next()
            .unwrap_or("")
            .trim()
            .trim_matches('"')
            .to_string();
        return Some(Primitive::Act {
            tool,
            params,
            sandbox: false,
        });
    }
    None
}

pub fn dispatch(runtime: &mut Option<AgentRuntime>, primitive: Primitive) -> String {
    match primitive {
        Primitive::Birth { name, .. } => {
            let created = AgentRuntime::birth(name);
            let summary = created.agent.hidden_state_summary();
            *runtime = Some(created);
            summary
        }
        Primitive::Think { prompt, private } => {
            let runtime = runtime.as_mut().expect("agent must be born first");
            runtime.think(&prompt, private)
        }
        Primitive::Act {
            tool,
            params,
            sandbox,
        } => {
            let runtime = runtime.as_mut().expect("agent must be born first");
            let receipt = runtime.act(&tool, &params, sandbox);
            serde_json::to_string(&receipt).unwrap_or_else(|_| "{}".to_string())
        }
    }
}

fn hex_hash(input: &str) -> String {
    let digest = Sha256::digest(input.as_bytes());
    digest
        .iter()
        .take(8)
        .map(|b| format!("{:02x}", b))
        .collect()
}
