pub const ACT_TIER_0_BASE: f64 = 0.040;

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
];
const TIER_3_TOOLS: &[&str] = &[
    "web_search",
    "note_taking",
    "read_file",
    "glob",
    "grep",
    "image_gen_basic",
    "code_runner",
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
    "data_analysis",
    "api_connect",
    "agent_orchestration",
    "self_modification",
    "multi_agent_fabric",
];

pub fn difficulty(reputation: f64) -> f64 {
    1.0 / (1.0 + (reputation / 25.0))
}

pub fn reputation_gain(base: f64, reputation: f64) -> f64 {
    base * difficulty(reputation)
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
