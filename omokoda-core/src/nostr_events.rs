/// Nostr event builders and publisher for sovereign agent lifecycle.
///
/// Used at birth (kind 0 profile), heartbeat (kind 30104), and future
/// lifecycle transitions. All publish calls are fire-and-forget —
/// relay unreachability never blocks agent operations.
///
/// Phase 8.1 — NIP-OSO-01/05 agent presence layer.

use serde_json::{json, Value};

// ── Odù name table (256 entries, index 0–255) ────────────────────────────────
// The canonical 16×16 grid: 16 major Odù × 16 sub-Odù.
// Kept inline so this module has zero dependency on omokoda-hermetic at link
// time — callers need only serde_json and tracing.
static ODU_NAMES: &[&str] = &[
    // Major 0 (Ogbe) — 0..15
    "Ogbe-Meji", "Ogbe-Oyeku", "Ogbe-Iwori", "Ogbe-Odi",
    "Ogbe-Irosun", "Ogbe-Owonrin", "Ogbe-Obara", "Ogbe-Okonron",
    "Ogbe-Ogunda", "Ogbe-Osa", "Ogbe-Ika", "Ogbe-Oturupon",
    "Ogbe-Otura", "Ogbe-Irete", "Ogbe-Ose", "Ogbe-Ofu",
    // Major 1 (Oyeku) — 16..31
    "Oyeku-Ogbe", "Oyeku-Meji", "Oyeku-Iwori", "Oyeku-Odi",
    "Oyeku-Irosun", "Oyeku-Owonrin", "Oyeku-Obara", "Oyeku-Okonron",
    "Oyeku-Ogunda", "Oyeku-Osa", "Oyeku-Ika", "Oyeku-Oturupon",
    "Oyeku-Otura", "Oyeku-Irete", "Oyeku-Ose", "Oyeku-Ofu",
    // Major 2 (Iwori) — 32..47
    "Iwori-Ogbe", "Iwori-Oyeku", "Iwori-Meji", "Iwori-Odi",
    "Iwori-Irosun", "Iwori-Owonrin", "Iwori-Obara", "Iwori-Okonron",
    "Iwori-Ogunda", "Iwori-Osa", "Iwori-Ika", "Iwori-Oturupon",
    "Iwori-Otura", "Iwori-Irete", "Iwori-Ose", "Iwori-Ofu",
    // Major 3 (Odi) — 48..63
    "Odi-Ogbe", "Odi-Oyeku", "Odi-Iwori", "Odi-Meji",
    "Odi-Irosun", "Odi-Owonrin", "Odi-Obara", "Odi-Okonron",
    "Odi-Ogunda", "Odi-Osa", "Odi-Ika", "Odi-Oturupon",
    "Odi-Otura", "Odi-Irete", "Odi-Ose", "Odi-Ofu",
    // Major 4 (Irosun) — 64..79
    "Irosun-Ogbe", "Irosun-Oyeku", "Irosun-Iwori", "Irosun-Odi",
    "Irosun-Meji", "Irosun-Owonrin", "Irosun-Obara", "Irosun-Okonron",
    "Irosun-Ogunda", "Irosun-Osa", "Irosun-Ika", "Irosun-Oturupon",
    "Irosun-Otura", "Irosun-Irete", "Irosun-Ose", "Irosun-Ofu",
    // Major 5 (Owonrin) — 80..95
    "Owonrin-Ogbe", "Owonrin-Oyeku", "Owonrin-Iwori", "Owonrin-Odi",
    "Owonrin-Irosun", "Owonrin-Meji", "Owonrin-Obara", "Owonrin-Okonron",
    "Owonrin-Ogunda", "Owonrin-Osa", "Owonrin-Ika", "Owonrin-Oturupon",
    "Owonrin-Otura", "Owonrin-Irete", "Owonrin-Ose", "Owonrin-Ofu",
    // Major 6 (Obara) — 96..111
    "Obara-Ogbe", "Obara-Oyeku", "Obara-Iwori", "Obara-Odi",
    "Obara-Irosun", "Obara-Owonrin", "Obara-Meji", "Obara-Okonron",
    "Obara-Ogunda", "Obara-Osa", "Obara-Ika", "Obara-Oturupon",
    "Obara-Otura", "Obara-Irete", "Obara-Ose", "Obara-Ofu",
    // Major 7 (Okonron) — 112..127
    "Okonron-Ogbe", "Okonron-Oyeku", "Okonron-Iwori", "Okonron-Odi",
    "Okonron-Irosun", "Okonron-Owonrin", "Okonron-Obara", "Okonron-Meji",
    "Okonron-Ogunda", "Okonron-Osa", "Okonron-Ika", "Okonron-Oturupon",
    "Okonron-Otura", "Okonron-Irete", "Okonron-Ose", "Okonron-Ofu",
    // Major 8 (Ogunda) — 128..143
    "Ogunda-Ogbe", "Ogunda-Oyeku", "Ogunda-Iwori", "Ogunda-Odi",
    "Ogunda-Irosun", "Ogunda-Owonrin", "Ogunda-Obara", "Ogunda-Okonron",
    "Ogunda-Meji", "Ogunda-Osa", "Ogunda-Ika", "Ogunda-Oturupon",
    "Ogunda-Otura", "Ogunda-Irete", "Ogunda-Ose", "Ogunda-Ofu",
    // Major 9 (Osa) — 144..159
    "Osa-Ogbe", "Osa-Oyeku", "Osa-Iwori", "Osa-Odi",
    "Osa-Irosun", "Osa-Owonrin", "Osa-Obara", "Osa-Okonron",
    "Osa-Ogunda", "Osa-Meji", "Osa-Ika", "Osa-Oturupon",
    "Osa-Otura", "Osa-Irete", "Osa-Ose", "Osa-Ofu",
    // Major 10 (Ika) — 160..175
    "Ika-Ogbe", "Ika-Oyeku", "Ika-Iwori", "Ika-Odi",
    "Ika-Irosun", "Ika-Owonrin", "Ika-Obara", "Ika-Okonron",
    "Ika-Ogunda", "Ika-Osa", "Ika-Meji", "Ika-Oturupon",
    "Ika-Otura", "Ika-Irete", "Ika-Ose", "Ika-Ofu",
    // Major 11 (Oturupon) — 176..191
    "Oturupon-Ogbe", "Oturupon-Oyeku", "Oturupon-Iwori", "Oturupon-Odi",
    "Oturupon-Irosun", "Oturupon-Owonrin", "Oturupon-Obara", "Oturupon-Okonron",
    "Oturupon-Ogunda", "Oturupon-Osa", "Oturupon-Ika", "Oturupon-Meji",
    "Oturupon-Otura", "Oturupon-Irete", "Oturupon-Ose", "Oturupon-Ofu",
    // Major 12 (Otura) — 192..207
    "Otura-Ogbe", "Otura-Oyeku", "Otura-Iwori", "Otura-Odi",
    "Otura-Irosun", "Otura-Owonrin", "Otura-Obara", "Otura-Okonron",
    "Otura-Ogunda", "Otura-Osa", "Otura-Ika", "Otura-Oturupon",
    "Otura-Meji", "Otura-Irete", "Otura-Ose", "Otura-Ofu",
    // Major 13 (Irete) — 208..223
    "Irete-Ogbe", "Irete-Oyeku", "Irete-Iwori", "Irete-Odi",
    "Irete-Irosun", "Irete-Owonrin", "Irete-Obara", "Irete-Okonron",
    "Irete-Ogunda", "Irete-Osa", "Irete-Ika", "Irete-Oturupon",
    "Irete-Otura", "Irete-Meji", "Irete-Ose", "Irete-Ofu",
    // Major 14 (Ose) — 224..239
    "Ose-Ogbe", "Ose-Oyeku", "Ose-Iwori", "Ose-Odi",
    "Ose-Irosun", "Ose-Owonrin", "Ose-Obara", "Ose-Okonron",
    "Ose-Ogunda", "Ose-Osa", "Ose-Ika", "Ose-Oturupon",
    "Ose-Otura", "Ose-Irete", "Ose-Meji", "Ose-Ofu",
    // Major 15 (Ofu/Ofun) — 240..255
    "Ofu-Ogbe", "Ofu-Oyeku", "Ofu-Iwori", "Ofu-Odi",
    "Ofu-Irosun", "Ofu-Owonrin", "Ofu-Obara", "Ofu-Okonron",
    "Ofu-Ogunda", "Ofu-Osa", "Ofu-Ika", "Ofu-Oturupon",
    "Ofu-Otura", "Ofu-Irete", "Ofu-Ose", "Ofu-Meji",
];

/// Return the canonical Odù name for an index (0–255).
/// Safe for any u8 — wraps to a valid entry.
pub fn odu_name_for(index: u8) -> &'static str {
    ODU_NAMES[index as usize % ODU_NAMES.len()]
}

// ── Event builders ────────────────────────────────────────────────────────────

/// Build a Nostr **kind 0** profile event (NIP-OSO-01).
///
/// The `npub` field here is the agent's hex public key (not bech32-encoded)
/// to keep this module dep-free. Relay clients encode to bech32 before display.
///
/// Content is a JSON-stringified NIP-01 metadata object. Tags carry sovereign
/// metadata: `bipon39`, `odu`, and `tier` so relay-side filters can query by
/// archetype or tier without decoding the content blob.
pub fn build_profile_event(
    npub: &str,
    bipon39_phrase: &str,
    odu_index: u8,
    tier: u8,
    vantage_host: &str,
    walrus_profile_url: Option<&str>,
) -> Value {
    let odu_name = odu_name_for(odu_index);
    let about = format!(
        "Odù: {} | BIPON39: {} | Tier: T{}",
        odu_name, bipon39_phrase, tier
    );
    let content_obj = json!({
        "name": bipon39_phrase,
        "about": about,
        "picture": walrus_profile_url.unwrap_or(""),
        "website": format!("https://{}/agents/{}/public", vantage_host, npub),
        "nip05": format!("{}@{}", &npub[..8.min(npub.len())], vantage_host),
    });
    let content_str =
        serde_json::to_string(&content_obj).unwrap_or_else(|_| "{}".to_string());

    json!({
        "kind": 0,
        "pubkey": npub,
        "content": content_str,
        "tags": [
            ["bipon39", bipon39_phrase],
            ["odu", odu_index.to_string()],
            ["tier", tier.to_string()],
        ]
    })
}

/// Build a **kind 30104** heartbeat event (NIP-OSO-05).
///
/// Uses a parameterised replaceable event (`d` = agent_id) so relays only
/// retain the latest heartbeat per agent. `chain_hash` is the blake3 hash of
/// the last receipt, forming a lightweight verifiable chain.
pub fn build_heartbeat_event(
    npub: &str,
    agent_id: &str,
    seq: u64,
    chain_hash: &str,
    lifecycle: &str,
) -> Value {
    json!({
        "kind": 30104,
        "pubkey": npub,
        "tags": [
            ["d", agent_id],
            ["seq", seq.to_string()],
            ["chain_hash", chain_hash],
            ["lifecycle", lifecycle],
        ],
        "content": ""
    })
}

/// Build a **kind 1** lifecycle transition note (NIP-OSO-06).
///
/// Used for human-readable milestone posts: first act, tier-up, fork, etc.
/// Tags include `t` (topic) for relay-side categorisation.
pub fn build_lifecycle_note(
    npub: &str,
    agent_id: &str,
    transition: &str,
    detail: &str,
) -> Value {
    json!({
        "kind": 1,
        "pubkey": npub,
        "content": format!("[{}] {} — {}", agent_id, transition, detail),
        "tags": [
            ["t", "sovereign-agent"],
            ["t", transition],
            ["agent_id", agent_id],
        ]
    })
}

/// Build a **kind 1** autonomous periodic status note.
///
/// Called by `lifecycle::nostr_publisher` every 6 hours (max 4/day).
/// The content is provided by the caller; this function just wraps it in
/// the standard NIP-01 envelope with sovereign-agent tags.
pub fn build_status_note_event(npub: &str, content: &str) -> Value {
    json!({
        "kind": 1,
        "pubkey": npub,
        "content": content,
        "tags": [
            ["t", "sovereign-agent"],
            ["t", "status"],
        ]
    })
}

// ── Publisher ─────────────────────────────────────────────────────────────────

/// Publish a Nostr event to all configured relays (fire-and-forget).
///
/// Signing and WebSocket relay transport are the full Phase 18 scope.
/// Until that wire implementation lands, this function logs intent and returns
/// `Ok(())` — callers spawn it with `tokio::spawn` so it never blocks birth.
///
/// `nsec_hex` — the agent's Nostr private key in hex (from IdentityVaultData).
/// `relay_list` — sourced from `IdentityVaultData::relay_list` or from the
///               `AGENT_NOSTR_RELAYS` env var (comma-separated URLs).
pub async fn publish_event(
    event: Value,
    nsec_hex: &str,
    relay_list: &[String],
) -> Result<(), String> {
    // Resolve relay list: if caller passes an empty slice, fall back to env var.
    let env_relays: Vec<String>;
    let effective_relays: &[String] = if relay_list.is_empty() {
        env_relays = std::env::var("AGENT_NOSTR_RELAYS")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        &env_relays
    } else {
        relay_list
    };

    if effective_relays.is_empty() {
        // No relays configured — this is expected on nodes without Nostr.
        // Do not log a warning; Phase 18 will make this a non-issue.
        return Ok(());
    }

    // Verify we have a usable nsec before attempting anything.
    if nsec_hex.is_empty() {
        tracing::debug!(
            kind = event["kind"].as_u64().unwrap_or(0),
            "nostr_events::publish_event: nsec_hex empty, skipping"
        );
        return Ok(());
    }

    // Phase 18 will replace this log with:
    //   1. secp256k1 sign (NIP-01 event id + schnorr sig)
    //   2. WS connect to each relay in parallel (tokio::select! with 5s timeout)
    //   3. Send ["EVENT", <signed_event>] JSON frame
    //   4. Wait for ["OK", ...] ack; log non-OK responses
    tracing::info!(
        kind = event["kind"].as_u64().unwrap_or(0),
        relay_count = effective_relays.len(),
        "nostr_events: would publish kind {} to {} relay(s) — relay WS impl pending (Phase 18)",
        event["kind"], effective_relays.len()
    );

    Ok(())
}

/// Convenience wrapper: publish a kind 0 profile event for a freshly-born agent.
///
/// Called from interpreter.rs birth step via `tokio::spawn`. Fails open.
pub async fn publish_birth_profile(
    npub: &str,
    nsec_hex: &str,
    bipon39_phrase: &str,
    odu_index: u8,
    tier: u8,
    relay_list: Vec<String>,
) {
    let vantage_host = std::env::var("VANTAGE_URL")
        .unwrap_or_default()
        .replace("http://", "")
        .replace("https://", "");
    let vantage_host = if vantage_host.is_empty() {
        "sovereign.local".to_string()
    } else {
        vantage_host
    };

    let event = build_profile_event(
        npub,
        bipon39_phrase,
        odu_index,
        tier,
        &vantage_host,
        None,
    );

    if let Err(e) = publish_event(event, nsec_hex, &relay_list).await {
        tracing::warn!("nostr birth profile publish failed (fail-open): {}", e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn odu_names_full_coverage() {
        // All 256 indices must resolve without panic.
        for i in 0u8..=255 {
            let name = odu_name_for(i);
            assert!(!name.is_empty(), "odu_name_for({i}) returned empty string");
        }
    }

    #[test]
    fn profile_event_kind_zero() {
        let ev = build_profile_event("deadbeef", "omi-eja-aye", 42, 2, "vantage.local", None);
        assert_eq!(ev["kind"], 0);
        assert_eq!(ev["pubkey"], "deadbeef");
        // Content is a JSON string
        let content: serde_json::Value =
            serde_json::from_str(ev["content"].as_str().unwrap()).unwrap();
        assert_eq!(content["name"], "omi-eja-aye");
    }

    #[test]
    fn heartbeat_event_kind_30104() {
        let ev = build_heartbeat_event("aabbcc", "agent-1", 7, "abc123", "ACTIVE");
        assert_eq!(ev["kind"], 30104);
        let tags = ev["tags"].as_array().unwrap();
        let d_tag = tags.iter().find(|t| t[0] == "d").unwrap();
        assert_eq!(d_tag[1], "agent-1");
    }

    #[test]
    fn profile_event_tags_present() {
        let ev = build_profile_event("pub123", "omi-eja", 10, 1, "h.local", Some("walrus://x"));
        let tags = ev["tags"].as_array().unwrap();
        let bipon_tag = tags.iter().find(|t| t[0] == "bipon39").unwrap();
        assert_eq!(bipon_tag[1], "omi-eja");
    }
}
