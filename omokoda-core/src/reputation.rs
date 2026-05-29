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
    if reputation < 80.0 {
        1.0 / (1.0 + (reputation / 25.0))
    } else {
        let bb_compression =
            (107.0 / 47_176_870.0_f64.powf((reputation - 80.0) / 20.0)).max(f64::EPSILON);
        1.0 / (1.0 + (reputation / 25.0)) * bb_compression
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn difficulty_bb_compression_above_80() {
        // Below 80: standard formula
        let d79 = difficulty(79.0);
        // At exactly 80.0: BB compression branch applies (condition is reputation < 80.0)
        // bb_compression = 107.0 / 47_176_870.0^0 = 107.0 / 1.0 = 107.0
        // d80 = (1/(1+80/25)) * 107 ≈ 0.238 * 107 ≈ 25.47 — much LARGER than d79
        let d80 = difficulty(80.0);
        // Above 80: BB compression decays — at rep=90: exponent=(90-80)/20=0.5
        // 47176870^0.5 ≈ 6870, bb_compression ≈ 0.0156
        // d90 = (1/(1+90/25)) * 0.0156 ≈ 0.217 * 0.0156 ≈ 0.0034
        let d90 = difficulty(90.0);
        let d99 = difficulty(99.0);
        // At the boundary (rep=80), BB compression kicks in at full strength (×107),
        // making difficulty LARGER than sub-80 values.
        assert!(
            d80 > d79,
            "BB compression jump makes difficulty larger at the 80 boundary"
        );
        // As rep rises above 80 the 47_176_870 denominator grows, collapsing difficulty.
        assert!(d90 < d80, "BB compression makes earning harder above 80");
        assert!(
            d99 < d90,
            "difficulty continues falling through sovereign range"
        );
    }
}
