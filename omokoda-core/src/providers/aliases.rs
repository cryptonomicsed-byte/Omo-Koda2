/// Model alias resolution — maps friendly shorthand to canonical provider model IDs.
/// Upgrades happen in one place, not scattered across configs.

/// Canonical Anthropic model IDs as of the current knowledge cutoff.
pub const ANTHROPIC_OPUS: &str = "claude-opus-4-7";
pub const ANTHROPIC_SONNET: &str = "claude-sonnet-4-6";
pub const ANTHROPIC_HAIKU: &str = "claude-haiku-4-5-20251001";

/// Resolve a user-supplied model name or alias to the canonical provider model ID.
/// Returns the input unchanged if no alias matches — providers receive it as-is.
///
/// Case-insensitive. Prefix matching handles partial names like "sonnet-4" or "haiku".
#[must_use]
pub fn resolve_model_alias(alias: &str) -> &'static str {
    let lower = alias.to_ascii_lowercase();
    // Exact aliases first
    match lower.as_str() {
        // Anthropic tiers
        "opus" | "claude-opus" | "claude-opus-4" | "claude-opus-4-7" => ANTHROPIC_OPUS,
        "sonnet" | "claude-sonnet" | "claude-sonnet-4" | "claude-sonnet-4-6" => ANTHROPIC_SONNET,
        "haiku" | "claude-haiku" | "claude-haiku-4" | "claude-haiku-4-5" => ANTHROPIC_HAIKU,
        // Omo-Koda2 sovereign tier aliases
        "fast" | "cheap" => ANTHROPIC_HAIKU,
        "balanced" | "default" => ANTHROPIC_SONNET,
        "powerful" | "smart" | "best" => ANTHROPIC_OPUS,
        _ => leak_str(alias),
    }
}

/// Returns true if `alias` is a known shorthand that would be rewritten.
#[must_use]
pub fn is_alias(alias: &str) -> bool {
    let resolved = resolve_model_alias(alias);
    resolved != alias
}

/// Infer the provider name from a model ID or alias.
#[must_use]
pub fn provider_for_model(model: &str) -> &'static str {
    let lower = model.to_ascii_lowercase();
    let resolved = resolve_model_alias(&lower);
    if resolved.starts_with("claude") || lower.contains("anthropic") {
        "anthropic"
    } else if lower.contains("gpt") || lower.contains("o1") || lower.contains("o3") {
        "openai"
    } else if lower.contains("gemini") {
        "google"
    } else {
        "ollama"
    }
}

fn leak_str(s: &str) -> &'static str {
    Box::leak(s.to_string().into_boxed_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opus_alias_resolves() {
        assert_eq!(resolve_model_alias("opus"), ANTHROPIC_OPUS);
        assert_eq!(resolve_model_alias("Opus"), ANTHROPIC_OPUS);
        assert_eq!(resolve_model_alias("best"), ANTHROPIC_OPUS);
    }

    #[test]
    fn sonnet_alias_resolves() {
        assert_eq!(resolve_model_alias("sonnet"), ANTHROPIC_SONNET);
        assert_eq!(resolve_model_alias("balanced"), ANTHROPIC_SONNET);
        assert_eq!(resolve_model_alias("default"), ANTHROPIC_SONNET);
    }

    #[test]
    fn haiku_alias_resolves() {
        assert_eq!(resolve_model_alias("haiku"), ANTHROPIC_HAIKU);
        assert_eq!(resolve_model_alias("fast"), ANTHROPIC_HAIKU);
        assert_eq!(resolve_model_alias("cheap"), ANTHROPIC_HAIKU);
    }

    #[test]
    fn unknown_alias_passes_through() {
        let custom = "llama3.1-70b";
        let resolved = resolve_model_alias(custom);
        assert_eq!(resolved, custom);
    }

    #[test]
    fn is_alias_detects_shorthand() {
        assert!(is_alias("opus"));
        assert!(is_alias("fast"));
        assert!(!is_alias("llama3.1-70b"));
    }

    #[test]
    fn provider_for_anthropic_model() {
        assert_eq!(provider_for_model("opus"), "anthropic");
        assert_eq!(provider_for_model(ANTHROPIC_SONNET), "anthropic");
    }

    #[test]
    fn provider_for_openai_model() {
        assert_eq!(provider_for_model("gpt-4o"), "openai");
        assert_eq!(provider_for_model("o1-preview"), "openai");
    }

    #[test]
    fn provider_for_unknown_falls_back_to_ollama() {
        assert_eq!(provider_for_model("mistral-7b"), "ollama");
    }
}
