//! ip-layer birth-flow hook — publishes this agent's IP Root (kind 31900,
//! see cryptonomicsed-byte/ip-layer's schemas/ip_root.md) at birth.
//!
//! Real, verified pattern: reuses identity/buzz_relay.rs's proven
//! nostr-sdk Client/EventBuilder/sign/send_event flow (already live for
//! NIP-29 group joins), and mirrors onchain.rs::mint_onchain_agent's
//! fail-open shape -- an unreachable relay or missing config never blocks
//! a birth, it just means no IP Root gets published this time.
//!
//! Signing key: this agent's own NIP-06 secp256k1 identity
//! (identity::nip06::derive_nip06_identity), the same derived key
//! nostr_identity_tool.rs already surfaces (npub only) to agent-visible
//! context -- the IP Root's pubkey is that same identity's public half,
//! so "who signed this" and "who the agent publicly is on Nostr" are the
//! same key, not a second unrelated one.
//!
//! `d` tag (the IP Root's stable addressable id): the agent's own hex
//! pubkey, per ip_root.md's own field note ("For Omo-Koda2-born agents
//! this can be the agent's own Nostr pubkey").

use nostr_sdk::prelude::*;
use serde_json::json;

use crate::identity::nip06::derive_nip06_identity;

/// Nostr kind 31900 — IP Root, per cryptonomicsed-byte/ip-layer's schema.
/// Provisional pending a real NIP submission (see that repo's own
/// open-questions note); this constant is the single source of truth for
/// the number within this kernel.
const IP_ROOT_KIND: u16 = 31900;

/// Publish this agent's IP Root at birth. Returns the real event id on
/// success, None on any failure (no relay configured, signing failed,
/// relay unreachable, etc.) -- never propagates an error, matching
/// onchain.rs::mint_onchain_agent's fail-open convention for optional
/// birth-time side effects.
///
/// `mnemonic`: the agent's own birth mnemonic, used to deterministically
/// derive the same NIP-06 identity nostr_identity_tool.rs exposes.
/// `agent_name`: for the event's free-form content field (display_name).
pub async fn publish_ip_root(mnemonic: &str, agent_name: &str) -> Option<String> {
    let relay_url = std::env::var("IP_LAYER_RELAY_URL")
        .or_else(|_| std::env::var("BUZZ_RELAY_URL"))
        .unwrap_or_else(|_| "ws://localhost:3000".to_string());

    let identity = derive_nip06_identity(mnemonic, 0).ok()?;
    let secret_key = SecretKey::from_hex(&identity.secret_key_hex).ok()?;
    let keys = Keys::new(secret_key);

    let content = json!({
        "display_name": agent_name,
        "framework": "omo-koda2",
    })
    .to_string();

    // `d` tag = this agent's own hex pubkey, per ip_root.md's own field
    // note for Omo-Koda2-born agents -- one IP Root per agent identity,
    // addressable/updatable in place (kind 31900 is a NIP-33 parameterized
    // replaceable range) rather than re-minting on every birth call.
    let event = EventBuilder::new(Kind::Custom(IP_ROOT_KIND), content)
        .tag(Tag::identifier(identity.public_key_hex.clone()))
        .sign(&keys)
        .await
        .ok()?;

    let client = Client::new(keys);
    client.add_relay(&relay_url).await.ok()?;
    client.connect().await;

    // Real bug found and fixed here (confirmed live via a direct probe
    // against an unreachable relay): send_event()'s outer Result is Ok(..)
    // even when EVERY relay in Output.failed rejected the event -- the
    // event only genuinely landed somewhere if Output.success is
    // non-empty. Checking only the outer Result (as this kernel's other
    // existing nostr-sdk callers currently do, e.g.
    // identity/buzz_relay.rs's self_join/join_and_chat_mentioning) would
    // silently report success for an event nobody ever received.
    let output = client.send_event(&event).await.ok();
    client.disconnect().await;

    match output {
        Some(o) if !o.success.is_empty() => Some(o.id().to_hex()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Explicit, fast-failing unreachable relay -- loopback with nothing
    /// bound (confirmed live via a direct probe: nostr-sdk's Client
    /// reports this as "relay not connected" within milliseconds, no
    /// timeout wait) -- NOT relying on "nothing happens to be listening on
    /// the ws://localhost:3000 default", which is environment-dependent
    /// and was confirmed false on this exact dev machine (a real local
    /// relay IS reachable there sometimes). Also the real regression test
    /// for the Output.success bug fixed above: before that fix, this
    /// exact scenario (event "sent" but accepted by zero relays) returned
    /// Some(event_id) instead of None.
    #[tokio::test]
    async fn publish_ip_root_fails_open_when_no_relay_accepts_the_event() {
        // SAFETY: test-only, single-threaded test binary; no other test in
        // this crate reads this env var.
        unsafe {
            std::env::set_var("IP_LAYER_RELAY_URL", "ws://127.0.0.1:19999");
        }
        let mnemonic = bipon39::entropy_to_mnemonic(&[9u8; 32])
            .expect("test entropy should produce a valid mnemonic")
            .join(" ");
        let result = publish_ip_root(&mnemonic, "test-agent").await;
        unsafe {
            std::env::remove_var("IP_LAYER_RELAY_URL");
        }
        assert!(result.is_none(), "expected None when no relay accepts the event, got {result:?}");
    }
}
