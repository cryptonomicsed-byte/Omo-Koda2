use chrono::{Datelike, Timelike, Utc, Weekday};
use serde_json::Value;

// ─── Kóòdù Daily Resonance ────────────────────────────────────────────────────

const KOODU_SUNDAY: &str = include_str!("koodu/sunday.json");
const KOODU_MONDAY: &str = include_str!("koodu/monday.json");
const KOODU_TUESDAY: &str = include_str!("koodu/tuesday.json");
const KOODU_WEDNESDAY: &str = include_str!("koodu/wednesday.json");
const KOODU_THURSDAY: &str = include_str!("koodu/thursday.json");
const KOODU_FRIDAY: &str = include_str!("koodu/friday.json");
const KOODU_SATURDAY: &str = include_str!("koodu/saturday.json");

/// Returns today's Kóòdù resonance JSON parsed as a serde_json::Value.
pub fn today_resonance() -> Value {
    let raw = match Utc::now().weekday() {
        Weekday::Sun => KOODU_SUNDAY,
        Weekday::Mon => KOODU_MONDAY,
        Weekday::Tue => KOODU_TUESDAY,
        Weekday::Wed => KOODU_WEDNESDAY,
        Weekday::Thu => KOODU_THURSDAY,
        Weekday::Fri => KOODU_FRIDAY,
        Weekday::Sat => KOODU_SATURDAY,
    };
    serde_json::from_str(raw).unwrap_or(serde_json::json!({"error": "parse failed"}))
}

/// Irreversible action categories that must pause on Sabbath.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionReversibility {
    Reversible,
    Irreversible,
}

/// Result of a rhythm gate check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RhythmDecision {
    Allow,
    /// Sabbath is active — action is queued, not denied.
    QueuedForSabbathEnd {
        reason: String,
    },
    /// Cooldown is active for this tool.
    Cooldown {
        remaining_secs: u64,
    },
}

pub struct RhythmGate;

impl RhythmGate {
    /// Returns true if it's currently the UTC Sabbath (Saturday).
    pub fn is_sabbath() -> bool {
        Utc::now().weekday() == Weekday::Sat
    }

    /// Returns the current UTC day name.
    pub fn current_day_name() -> &'static str {
        match Utc::now().weekday() {
            Weekday::Sun => "Sunday",
            Weekday::Mon => "Monday",
            Weekday::Tue => "Tuesday",
            Weekday::Wed => "Wednesday",
            Weekday::Thu => "Thursday",
            Weekday::Fri => "Friday",
            Weekday::Sat => "Saturday",
        }
    }

    /// Returns seconds remaining in the current Sabbath (if active), else 0.
    pub fn sabbath_seconds_remaining() -> u64 {
        if !Self::is_sabbath() {
            return 0;
        }
        let now = Utc::now();
        // Sabbath ends at midnight Saturday → Sunday UTC
        let secs_into_day = (now.num_seconds_from_midnight()) as u64;
        86_400u64.saturating_sub(secs_into_day)
    }

    /// Gate an action based on reversibility and current rhythm state.
    pub fn check(
        action: &str,
        reversibility: ActionReversibility,
        cooldown_remaining_secs: u64,
    ) -> RhythmDecision {
        if cooldown_remaining_secs > 0 {
            return RhythmDecision::Cooldown {
                remaining_secs: cooldown_remaining_secs,
            };
        }
        if reversibility == ActionReversibility::Irreversible && Self::is_sabbath() {
            return RhythmDecision::QueuedForSabbathEnd {
                reason: format!(
                    "Action '{}' is irreversible. Sabbath is active (UTC Saturday). \
                     This action will execute when Sabbath ends. \
                     {} seconds remaining.",
                    action,
                    Self::sabbath_seconds_remaining()
                ),
            };
        }
        RhythmDecision::Allow
    }

    /// Classify whether a tool action is irreversible.
    pub fn classify_reversibility(tool: &str) -> ActionReversibility {
        match tool {
            "write_file"
            | "delete_file"
            | "bash"
            | "api_connect"
            | "agent_orchestration"
            | "self_modification"
            // zero patch mutates the agent's own program graph — the purest
            // form of self-modification. Queued on the Sabbath while the
            // dream engine runs its REM cycle.
            | "zero"
            | "multi_agent_fabric" => ActionReversibility::Irreversible,
            _ => ActionReversibility::Reversible,
        }
    }
}

/// Per-agent per-tool cooldown tracker (in-memory).
#[derive(Debug, Clone, Default)]
pub struct CooldownTracker {
    // (tool_name, expiry_unix_timestamp)
    cooldowns: Vec<(String, u64)>,
}

impl CooldownTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a cooldown for `tool` lasting `duration_secs` from now.
    pub fn set(&mut self, tool: &str, duration_secs: u64) {
        let expiry = current_unix_timestamp() + duration_secs;
        if let Some(entry) = self.cooldowns.iter_mut().find(|(t, _)| t == tool) {
            entry.1 = expiry;
        } else {
            self.cooldowns.push((tool.to_string(), expiry));
        }
    }

    /// Returns remaining cooldown seconds for `tool`, or 0 if none.
    pub fn remaining(&self, tool: &str) -> u64 {
        let now = current_unix_timestamp();
        self.cooldowns
            .iter()
            .find(|(t, _)| t == tool)
            .map(|(_, expiry)| expiry.saturating_sub(now))
            .unwrap_or(0)
    }

    /// Removes expired cooldowns.
    pub fn prune(&mut self) {
        let now = current_unix_timestamp();
        self.cooldowns.retain(|(_, expiry)| *expiry > now);
    }
}

fn current_unix_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_file_is_irreversible() {
        assert_eq!(
            RhythmGate::classify_reversibility("write_file"),
            ActionReversibility::Irreversible
        );
    }

    #[test]
    fn web_search_is_reversible() {
        assert_eq!(
            RhythmGate::classify_reversibility("web_search"),
            ActionReversibility::Reversible
        );
    }

    #[test]
    fn cooldown_remaining_zero_when_none() {
        let tracker = CooldownTracker::new();
        assert_eq!(tracker.remaining("some_tool"), 0);
    }

    #[test]
    fn cooldown_set_and_active() {
        let mut tracker = CooldownTracker::new();
        tracker.set("bash", 60);
        assert!(tracker.remaining("bash") > 0);
        assert!(tracker.remaining("bash") <= 60);
    }

    #[test]
    fn cooldown_triggers_gate() {
        let decision = RhythmGate::check("bash", ActionReversibility::Reversible, 30);
        assert!(matches!(
            decision,
            RhythmDecision::Cooldown { remaining_secs: 30 }
        ));
    }

    #[test]
    fn reversible_action_allowed_any_day() {
        // web_search is reversible — should always be allowed (no cooldown)
        let decision = RhythmGate::check("web_search", ActionReversibility::Reversible, 0);
        assert_eq!(decision, RhythmDecision::Allow);
    }

    #[test]
    fn day_name_is_valid() {
        let day = RhythmGate::current_day_name();
        let valid = [
            "Sunday",
            "Monday",
            "Tuesday",
            "Wednesday",
            "Thursday",
            "Friday",
            "Saturday",
        ];
        assert!(valid.contains(&day));
    }
}
