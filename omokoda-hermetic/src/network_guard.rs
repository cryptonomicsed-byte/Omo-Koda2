use serde::{Deserialize, Serialize};

/// Decision from the network guard
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkGuardDecision {
    Allow,
    Warn(String),
    Block(String),
}

impl NetworkGuardDecision {
    pub fn is_blocked(&self) -> bool {
        matches!(self, Self::Block(_))
    }
}

/// Configuration for the network guard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkGuardConfig {
    /// Block RFC-1918 / link-local / loopback targets
    pub block_private_ranges: bool,
    /// Block cloud metadata endpoints (169.254.169.254, etc.)
    pub block_metadata_endpoints: bool,
    /// Explicit allowlist of domains (empty = allow all non-blocked)
    pub domain_allowlist: Vec<String>,
    /// Explicit blocklist of domains
    pub domain_blocklist: Vec<String>,
}

impl Default for NetworkGuardConfig {
    fn default() -> Self {
        Self {
            block_private_ranges: true,
            block_metadata_endpoints: true,
            domain_allowlist: Vec::new(),
            domain_blocklist: Vec::new(),
        }
    }
}

/// SSRF Guard — validates network-bound requests against known attack patterns.
/// Ports `ssrfGuard.ts`.
pub struct NetworkGuard {
    config: NetworkGuardConfig,
}

impl Default for NetworkGuard {
    fn default() -> Self {
        Self::new(NetworkGuardConfig::default())
    }
}

impl NetworkGuard {
    pub fn new(config: NetworkGuardConfig) -> Self {
        Self { config }
    }

    /// Check a URL or hostname for SSRF risk.
    /// Returns Allow, Warn, or Block with a reason.
    pub fn check(&self, url: &str) -> NetworkGuardDecision {
        let normalized = url.trim().to_lowercase();

        // 1. Domain blocklist (exact + suffix match)
        if let Some(reason) = self.check_domain_blocklist(&normalized) {
            return NetworkGuardDecision::Block(reason);
        }

        // 2. Cloud metadata endpoints (highest priority block)
        if self.config.block_metadata_endpoints {
            if let Some(reason) = self.check_metadata_endpoints(&normalized) {
                return NetworkGuardDecision::Block(reason);
            }
        }

        // 3. Private/loopback IP ranges
        if self.config.block_private_ranges {
            if let Some(reason) = self.check_private_ranges(&normalized) {
                return NetworkGuardDecision::Block(reason);
            }
        }

        // 4. Suspicious redirect indicators
        if let Some(reason) = self.check_redirect_indicators(&normalized) {
            return NetworkGuardDecision::Warn(reason);
        }

        // 5. Domain allowlist enforcement
        if !self.config.domain_allowlist.is_empty() {
            let host = extract_host(&normalized);
            let allowed = self
                .config
                .domain_allowlist
                .iter()
                .any(|allowed| host_matches(host, allowed));
            if !allowed {
                return NetworkGuardDecision::Block(format!(
                    "SSRF guard: '{}' not in domain allowlist",
                    host
                ));
            }
        }

        NetworkGuardDecision::Allow
    }

    fn check_metadata_endpoints(&self, url: &str) -> Option<String> {
        let metadata_patterns = [
            "169.254.169.254", // AWS/GCP/Azure IMDS
            "metadata.google.internal",
            "169.254.170.2",   // ECS task metadata
            "100.100.100.200", // Alibaba Cloud IMDS
            "192.0.0.192",     // Oracle Cloud IMDS
        ];
        for pattern in &metadata_patterns {
            if url.contains(pattern) {
                return Some(format!(
                    "SSRF guard: blocked cloud metadata endpoint '{}'",
                    pattern
                ));
            }
        }
        None
    }

    fn check_private_ranges(&self, url: &str) -> Option<String> {
        let host = extract_host(url);

        // Loopback
        if host == "localhost" || host == "::1" || host.starts_with("127.") || host == "[::1]" {
            return Some(format!("SSRF guard: loopback address blocked '{}'", host));
        }

        // RFC-1918 private ranges
        if let Some(reason) = check_rfc1918(host) {
            return Some(reason);
        }

        // Link-local
        if host.starts_with("169.254.") {
            return Some(format!("SSRF guard: link-local address blocked '{}'", host));
        }

        // IPv6 private/link-local
        if host.starts_with("fc") || host.starts_with("fd") || host.starts_with("fe80") {
            return Some(format!(
                "SSRF guard: IPv6 private/link-local blocked '{}'",
                host
            ));
        }

        None
    }

    fn check_domain_blocklist(&self, url: &str) -> Option<String> {
        let host = extract_host(url);
        for blocked in &self.config.domain_blocklist {
            if host_matches(host, blocked) {
                return Some(format!("SSRF guard: domain '{}' is in blocklist", host));
            }
        }
        None
    }

    fn check_redirect_indicators(&self, url: &str) -> Option<String> {
        // Detect URL redirectors that might bypass host checks
        let suspicious = [
            "redirect=",
            "next=",
            "url=",
            "target=",
            "dest=",
            "forward=",
            "return=",
        ];
        for indicator in &suspicious {
            if url.contains(indicator) {
                return Some(format!(
                    "SSRF guard: redirect parameter '{}' may enable SSRF",
                    indicator
                ));
            }
        }

        // Detect double-encoded slashes or protocol confusion
        if url.contains("@") && url.contains("://") {
            return Some("SSRF guard: URL with credentials may obscure actual target".to_string());
        }

        None
    }
}

/// Extract the host portion from a URL string (best-effort, no full URL parsing)
fn extract_host(url: &str) -> &str {
    // Strip scheme
    let after_scheme = if let Some(pos) = url.find("://") {
        &url[pos + 3..]
    } else {
        url
    };

    // Strip path, query, fragment
    let host_port = after_scheme
        .split('/')
        .next()
        .unwrap_or(after_scheme)
        .split('?')
        .next()
        .unwrap_or(after_scheme)
        .split('#')
        .next()
        .unwrap_or(after_scheme);

    // Strip port
    // Handle IPv6 [::1]:port
    if host_port.starts_with('[') {
        if let Some(end) = host_port.find(']') {
            return &host_port[..=end];
        }
    }
    host_port.split(':').next().unwrap_or(host_port)
}

/// Check if a host matches an allowlist/blocklist entry (exact or suffix `.example.com`)
fn host_matches(host: &str, pattern: &str) -> bool {
    if host == pattern {
        return true;
    }
    // Suffix match: pattern ".example.com" matches "api.example.com"
    if let Some(suffix) = pattern.strip_prefix('.') {
        return host == suffix || host.ends_with(&format!(".{}", suffix));
    }
    // Pattern without leading dot: exact or subdomain
    host == pattern || host.ends_with(&format!(".{}", pattern))
}

/// Check RFC-1918 private IPv4 ranges
fn check_rfc1918(host: &str) -> Option<String> {
    if host.starts_with("10.") {
        return Some(format!(
            "SSRF guard: RFC-1918 10.0.0.0/8 blocked '{}'",
            host
        ));
    }
    if host.starts_with("192.168.") {
        return Some(format!(
            "SSRF guard: RFC-1918 192.168.0.0/16 blocked '{}'",
            host
        ));
    }
    // 172.16.0.0/12 = 172.16.x.x through 172.31.x.x
    if let Some(rest) = host.strip_prefix("172.") {
        if let Some(second_octet) = rest.split('.').next() {
            if let Ok(n) = second_octet.parse::<u8>() {
                if (16..=31).contains(&n) {
                    return Some(format!(
                        "SSRF guard: RFC-1918 172.16.0.0/12 blocked '{}'",
                        host
                    ));
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn guard() -> NetworkGuard {
        NetworkGuard::default()
    }

    #[test]
    fn test_block_localhost() {
        assert!(guard().check("http://localhost/api").is_blocked());
        assert!(guard().check("http://127.0.0.1/admin").is_blocked());
    }

    #[test]
    fn test_block_metadata() {
        assert!(guard()
            .check("http://169.254.169.254/latest/meta-data/")
            .is_blocked());
        assert!(guard()
            .check("http://metadata.google.internal/computeMetadata/v1/")
            .is_blocked());
    }

    #[test]
    fn test_block_private_ranges() {
        assert!(guard().check("http://10.0.0.1/secret").is_blocked());
        assert!(guard().check("http://192.168.1.1/").is_blocked());
        assert!(guard().check("http://172.16.0.1/").is_blocked());
        assert!(guard().check("http://172.31.255.255/").is_blocked());
        assert!(!guard().check("http://172.32.0.1/").is_blocked());
    }

    #[test]
    fn test_allow_public_url() {
        assert_eq!(
            guard().check("https://api.example.com/v1"),
            NetworkGuardDecision::Allow
        );
    }

    #[test]
    fn test_allowlist_enforcement() {
        let guard = NetworkGuard::new(NetworkGuardConfig {
            domain_allowlist: vec!["api.example.com".to_string()],
            ..Default::default()
        });
        assert_eq!(
            guard.check("https://api.example.com/v1"),
            NetworkGuardDecision::Allow
        );
        assert!(guard.check("https://evil.com/").is_blocked());
    }

    #[test]
    fn test_redirect_param_warns() {
        let decision = guard().check("https://safe.com/?redirect=http://evil.com");
        assert!(matches!(decision, NetworkGuardDecision::Warn(_)));
    }

    #[test]
    fn test_blocklist() {
        let guard = NetworkGuard::new(NetworkGuardConfig {
            domain_blocklist: vec!["evil.com".to_string()],
            ..Default::default()
        });
        assert!(guard.check("https://evil.com/payload").is_blocked());
        assert!(guard.check("https://sub.evil.com/payload").is_blocked());
    }
}
