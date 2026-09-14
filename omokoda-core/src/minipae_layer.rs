//! minipae NIP-AE birth-write hook — publishes a NIP-AE kind 30078 event
//! under the agent's minipae identity at birth.
//!
//! Pattern mirrors ip_layer::publish_ip_root: fail-open, never blocks birth,
//! uses the same sign_and_publish convention.
//!
//! Relay: MINIPAE_RELAY_URL → BUZZ_RELAY_URL → localhost fallback.
//! Key: agent's minipae secp256k1 key derived from the mnemonic
//! (m/44'/30174'/<agent_index>'/<owner_index>') via identity::wallet.

use nostr_sdk::prelude::*;
use serde_json::json;

use crate::identity::wallet::derive_minipae_key;

/// NIP-AE addressable application data kind.
const MINIPAE_BIRTH_KIND: u16 = 30078;

fn relay_url() -> String {
    std::env::var("MINIPAE_RELAY_URL")
        .or_else(|_| std::env::var("BUZZ_RELAY_URL"))
        .unwrap_or_else(|_| "ws://localhost:3000".to_string())
}

async fn sign_and_publish(keys: Keys, builder: EventBuilder) -> Option<String> {
    let event = builder.sign(&keys).await.ok()?;

    let client = Client::new(keys);
    client.add_relay(&relay_url()).await.ok()?;
    client.connect().await;

    let output = client.send_event(&event).await.ok();
    client.disconnect().await;

    match output {
        Some(o) if !o.success.is_empty() => Some(o.id().to_hex()),
        _ => None,
    }
}

/// Publish the agent's minipae birth event (kind 30078) at birth.
/// Returns the real event id on success, None on any failure — fail-open.
///
/// `mnemonic`: the agent's own birth mnemonic.
/// `agent_name`: display name for the `d` tag / content payload.
/// `genesis_receipt_id`: genesis receipt id to embed as provenance.
pub async fn publish_minipae_birth(
    mnemonic: &str,
    agent_name: &str,
    genesis_receipt_id: &str,
) -> Option<String> {
    let minipae = derive_minipae_key(mnemonic, "", 0, 0).ok()?;
    let secret_key = SecretKey::from_hex(&minipae.private_key_hex).ok()?;
    let keys = Keys::new(secret_key);

    let content = json!({
        "kind": "birth",
        "path": "mem/birth/genesis",
        "agent_name": agent_name,
        "genesis_receipt_id": genesis_receipt_id,
        "framework": "omo-koda2",
    })
    .to_string();

    let builder = EventBuilder::new(Kind::Custom(MINIPAE_BIRTH_KIND), content)
        .tag(Tag::identifier(format!("birth:{agent_name}")))
        .tag(Tag::custom(TagKind::custom("genesis"), vec![genesis_receipt_id.to_string()]));

    sign_and_publish(keys, builder).await
}
