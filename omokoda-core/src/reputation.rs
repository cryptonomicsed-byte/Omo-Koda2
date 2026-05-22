use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const THINK_NORMAL: f64 = 0.008;
pub const THINK_HIGH: f64 = 0.020;
pub const ACT_TIER_0: f64 = 0.040;
pub const ACT_TIER_1: f64 = 0.060;
pub const ACT_TIER_2: f64 = 0.100;
pub const ACT_TIER_3: f64 = 0.140;
pub const ACT_TIER_4: f64 = 0.180;

pub const DECAY_DAILY: f64 = -0.008;
pub const DECAY_EXTENDED: f64 = -0.015;
pub const SANDBOX_DECAY: f64 = -0.010;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReputationChangeReason {
    Think,
    Act,
    Decay,
    Violation,
    BudgetOverrun,
    ManualAudit,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReputationEntry {
    pub timestamp: u64,
    pub amount: f64,
    pub reason: ReputationChangeReason,
    pub previous_reputation: f64,
    pub new_reputation: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ReputationLedger {
    pub entries: Vec<ReputationEntry>,
}

impl ReputationLedger {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn record(&mut self, entry: ReputationEntry) {
        self.entries.push(entry);
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PermissionMode {
    ReadOnly,
    WorkspaceWrite,
    DangerFullAccess,
    Prompt,
    Allow,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PermissionPolicy {
    pub active_mode: PermissionMode,
}

impl Default for PermissionPolicy {
    fn default() -> Self {
        Self {
            active_mode: PermissionMode::ReadOnly,
        }
    }
}

pub fn mode_for_tier(tier: u8) -> PermissionMode {
    match tier {
        0 => PermissionMode::ReadOnly,
        1 | 2 => PermissionMode::WorkspaceWrite,
        3 | 4 => PermissionMode::Prompt,
        _ => PermissionMode::Allow,
    }
}

const TIER_0_TOOLS: &[&str] = &["web_search", "note_taking", "read_file", "glob", "grep"];
const TIER_1_TOOLS: &[&str] = &[
    "web_search",
    "note_taking",
    "read_file",
    "glob",
    "grep",
    "image_gen_basic",
];
const TIER_2_TOOLS: &[&str] = &[
    "web_search",
    "note_taking",
    "read_file",
    "glob",
    "grep",
    "image_gen_basic",
    "code_runner",
    "bash",
];
const TIER_3_TOOLS: &[&str] = &[
    "web_search",
    "note_taking",
    "read_file",
    "glob",
    "grep",
    "image_gen_basic",
    "code_runner",
    "bash",
    "data_analysis",
    "api_connect",
];
const TIER_4_TOOLS: &[&str] = &[
    "web_search",
    "note_taking",
    "read_file",
    "glob",
    "grep",
    "image_gen_basic",
    "code_runner",
    "bash",
    "data_analysis",
    "api_connect",
    "agent_orchestration",
];
const TIER_5_TOOLS: &[&str] = &[
    "web_search",
    "note_taking",
    "read_file",
    "glob",
    "grep",
    "image_gen_basic",
    "code_runner",
    "bash",
    "data_analysis",
    "api_connect",
    "agent_orchestration",
    "self_modification",
    "multi_agent_fabric",
];

pub fn difficulty(reputation: f64) -> f64 {
    1.0 / (1.0 + (reputation / 25.0))
}

pub fn reputation_gain(base: f64, reputation: f64, multiplier: f64) -> f64 {
    base * difficulty(reputation) * multiplier
}

pub fn tier_for(reputation: f64) -> u8 {
    if reputation >= 100.0 {
        5
    } else if reputation > 80.0 {
        4
    } else if reputation > 60.0 {
        3
    } else if reputation > 40.0 {
        2
    } else if reputation > 20.0 {
        1
    } else {
        0
    }
}

pub fn tools_for_tier(tier: u8) -> Vec<String> {
    tool_slice_for_tier(tier)
        .iter()
        .map(|tool| (*tool).to_string())
        .collect()
}

pub fn tool_allowed(tier: u8, tool: &str) -> bool {
    tool_slice_for_tier(tier).contains(&tool)
}

fn tool_slice_for_tier(tier: u8) -> &'static [&'static str] {
    match tier {
        0 => TIER_0_TOOLS,
        1 => TIER_1_TOOLS,
        2 => TIER_2_TOOLS,
        3 => TIER_3_TOOLS,
        4 => TIER_4_TOOLS,
        _ => TIER_5_TOOLS,
    }
}

// --- Safeguards added to support daily action caps and 7-day tier promotion gates ---

pub const MAX_ACTIONS_PER_DAY: u32 = 50;
pub const MIN_DAYS_BETWEEN_PROMOTIONS: u64 = 7;
pub const DIMINISHING_RETURNS_BASE: f64 = 0.995;

/// Returns the multiplier applied to reputation gain based on how many actions
/// the agent has already taken today. The 50th action gets ~0.778x; beyond the
/// cap (`MAX_ACTIONS_PER_DAY`) the gain is 0.0.
pub fn daily_gain_multiplier(actions_today: u32) -> f64 {
    if actions_today >= MAX_ACTIONS_PER_DAY {
        return 0.0;
    }
    DIMINISHING_RETURNS_BASE.powi(actions_today as i32)
}

/// Returns true if the agent is eligible for tier promotion.
/// An agent that has never been promoted is always eligible.
/// After a promotion, the agent must wait at least 7 days before the next one.
pub fn can_promote_tier(last_promotion: Option<DateTime<Utc>>) -> bool {
    match last_promotion {
        None => true,
        Some(ts) => {
            let days = (Utc::now() - ts).num_days() as u64;
            days >= MIN_DAYS_BETWEEN_PROMOTIONS
        }
    }
}
