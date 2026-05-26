use chrono::{Datelike, Utc, Weekday};
use serde::{Deserialize, Serialize};

/// Policy for Sabbath gate: which operations require Sunday confirmation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SabbathPolicy {
    /// Runs immediately, no day restriction.
    Unrestricted,
    /// Queued until Sunday (Ọbàtálá's day) — irreversible soul mutations only.
    SundayOnly,
    /// Permanently forbidden regardless of day.
    Forbidden,
}

/// Sabbath Gate — guards irreversible operations.
/// Governance constraint, not a UX feature. Pre-mainnet requirement.
pub struct SabbathGate;

impl SabbathGate {
    /// Returns true if the operation is permitted to execute right now.
    pub fn is_permitted(policy: &SabbathPolicy) -> bool {
        match policy {
            SabbathPolicy::Unrestricted => true,
            SabbathPolicy::Forbidden => false,
            SabbathPolicy::SundayOnly => {
                let today = Utc::now().weekday();
                today == Weekday::Sun
            }
        }
    }

    /// Returns the reason for blocking, if blocked.
    pub fn block_reason(policy: &SabbathPolicy) -> Option<&'static str> {
        match policy {
            SabbathPolicy::Unrestricted => None,
            SabbathPolicy::Forbidden => Some("operation permanently forbidden"),
            SabbathPolicy::SundayOnly => {
                let today = Utc::now().weekday();
                if today == Weekday::Sun {
                    None
                } else {
                    Some("irreversible soul mutation queued for Ọbàtálá's day (Sunday)")
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unrestricted_always_permitted() {
        assert!(SabbathGate::is_permitted(&SabbathPolicy::Unrestricted));
        assert!(SabbathGate::block_reason(&SabbathPolicy::Unrestricted).is_none());
    }

    #[test]
    fn forbidden_never_permitted() {
        assert!(!SabbathGate::is_permitted(&SabbathPolicy::Forbidden));
        assert!(SabbathGate::block_reason(&SabbathPolicy::Forbidden).is_some());
    }

    #[test]
    fn sunday_only_has_reason_on_non_sunday() {
        // We can't control what day the test runs, but we can test the logic
        let today = Utc::now().weekday();
        let permitted = SabbathGate::is_permitted(&SabbathPolicy::SundayOnly);
        let has_reason = SabbathGate::block_reason(&SabbathPolicy::SundayOnly).is_some();
        if today == Weekday::Sun {
            assert!(permitted);
            assert!(!has_reason);
        } else {
            assert!(!permitted);
            assert!(has_reason);
        }
    }
}
