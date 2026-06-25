use chrono::{Datelike, Utc, Weekday};
use serde::{Deserialize, Serialize};

/// Policy for Sabbath gate: which operations require Saturday confirmation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SabbathPolicy {
    /// Runs immediately, no day restriction.
    Unrestricted,
    /// Queued until Saturday (Ọbàtálá's day) — irreversible soul mutations only.
    SaturdayOnly,
    /// Permanently forbidden regardless of day.
    Forbidden,
}

/// Sabbath Gate — guards irreversible operations.
pub struct SabbathGate;

impl SabbathGate {
    pub fn is_permitted(policy: &SabbathPolicy) -> bool {
        match policy {
            SabbathPolicy::Unrestricted => true,
            SabbathPolicy::Forbidden => false,
            SabbathPolicy::SaturdayOnly => {
                let today = Utc::now().weekday();
                today == Weekday::Sat
            }
        }
    }

    pub fn block_reason(policy: &SabbathPolicy) -> Option<&'static str> {
        match policy {
            SabbathPolicy::Unrestricted => None,
            SabbathPolicy::Forbidden => Some("operation permanently forbidden"),
            SabbathPolicy::SaturdayOnly => {
                let today = Utc::now().weekday();
                if today == Weekday::Sat {
                    None
                } else {
                    Some("irreversible soul mutation queued for Ọbàtálá's day (Saturday)")
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
        let today = Utc::now().weekday();
        let permitted = SabbathGate::is_permitted(&SabbathPolicy::SaturdayOnly);
        let has_reason = SabbathGate::block_reason(&SabbathPolicy::SaturdayOnly).is_some();
        if today == Weekday::Sat {
            assert!(permitted);
            assert!(!has_reason);
        } else {
            assert!(!permitted);
            assert!(has_reason);
        }
    }
}
