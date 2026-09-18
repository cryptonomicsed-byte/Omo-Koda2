use super::receipt::AgentGenesisReceipt;
use serde::{Deserialize, Serialize};

/// Canonical public manifest of all network bindings and identity commitments.
/// Derived at birth from AgentGenesisReceipt. Updated as new bindings are
/// established (IP-Layer, device, federation, economy). Never contains secrets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct AgentManifest {
    // ── Core Identity ─────────────────────────────────────────────────
    pub identity: IdentitySection,

    // ── Genesis provenance ────────────────────────────────────────────
    pub genesis: GenesisSection,

    // ── Temporal anchor ───────────────────────────────────────────────
    pub temporal: TemporalSection,

    // ── Soul / semantic genome ────────────────────────────────────────
    pub soul: SoulSection,

    // ── Memory namespace ──────────────────────────────────────────────
    pub memory: MemorySection,

    // ── Network addresses (all are manifestations of the same identity)
    pub network: NetworkSection,

    // ── Economic bindings ─────────────────────────────────────────────
    pub economic: EconomicSection,

    // ── Physical body ─────────────────────────────────────────────────
    pub body: BodySection,

    // ── World / simulation bindings ───────────────────────────────────
    pub world: WorldSection,

    // ── Proofs, receipts, witnesses ───────────────────────────────────
    pub proof: ProofSection,

    /// Manifest version for forward compat
    pub manifest_version: u8,
}

impl AgentManifest {
    pub const CURRENT_VERSION: u8 = 1;

    /// Derive an AgentManifest from a completed AgentGenesisReceipt.
    pub fn from_genesis(gr: &AgentGenesisReceipt) -> Self {
        Self {
            identity: IdentitySection {
                agent_id: gr.agent_id.clone(),
                genesis_id: gr.genesis_hash.clone(),
                identity_commitment: gr.harmonic_signature.clone(),
                capability_root: gr.derivation_root_id.clone(),
                birth_receipt_hash: gr.genesis_hash.clone(),
                owner: None,
            },
            genesis: GenesisSection {
                bipon39_commitment: gr.birth_entropy_commitment.clone(),
                harmonic_signature: gr.harmonic_signature.clone(),
                genesis_receipt_hash: gr.genesis_hash.clone(),
                sigil_hash: gr.sigil_hash.clone(),
                symbolic_address: gr.symbolic_address.clone(),
            },
            temporal: TemporalSection {
                born_at: gr.born_at,
                koodu_epoch: gr.koodu_epoch,
                koodu_cycle: gr.koodu_cycle,
                koodu_phase: gr.koodu_phase,
                btc_height: gr.bitcoin_height,
                btc_anchor: gr.bitcoin_anchor.clone(),
                gregorian_fallback: gr.gregorian_fallback,
            },
            soul: SoulSection {
                primary_odu: gr.primary_odu,
                composed_odu: gr.composed_odu,
                temperament: gr.temperament.clone(),
                orisha_alignment: gr.orisha_alignment.clone(),
                destiny_threads: gr.destiny_threads.clone(),
                hermetic_fingerprint: gr.hermetic_fingerprint.clone(),
            },
            memory: MemorySection {
                minipae_pubkey: gr.minipae_pubkey.clone(),
                memory_root: gr.memory_root.clone(),
                glyph_root: gr.birth_memory_glyph.clone(),
                birth_glyph: gr.birth_memory_glyph.clone(),
            },
            network: NetworkSection {
                ip_root_event: gr.ip_root_event.clone(),
                nostr_pubkey: None,
                sui_address: None,
                btc_address: None,
                eth_address: None,
                mesh_id: None,
                freenet_key: None,
                web2_bindings: vec![],
            },
            economic: EconomicSection {
                sui_object_id: None,
                vantage_key: None,
                ase_balance: None,
                reputation: 0.0,
                wallet_bindings: vec![],
            },
            body: BodySection {
                device_binding: gr.device_binding.as_ref().map(|d| DeviceRef {
                    device_id: d.device_id.clone(),
                    device_kind: d.device_kind.clone(),
                    bound_at: d.binding_timestamp,
                }),
                twin_id: None,
                additional_devices: vec![],
            },
            world: WorldSection {
                osovm_endpoint: None,
                zvm_endpoint: None,
                twin_root: None,
            },
            proof: ProofSection {
                genesis_signature: gr.genesis_signature.clone(),
                witness_receipt: gr.witness_receipt.clone(),
                receipt_chain: vec![],
                anchors: vec![],
            },
            manifest_version: Self::CURRENT_VERSION,
        }
    }

    /// Bind a Nostr pubkey to the network section.
    pub fn bind_nostr(&mut self, pubkey: String) {
        self.network.nostr_pubkey = Some(pubkey);
    }

    /// Bind a Sui address to the network and economic sections.
    pub fn bind_sui(&mut self, address: String, object_id: Option<String>) {
        self.network.sui_address = Some(address);
        if let Some(id) = object_id {
            self.economic.sui_object_id = Some(id);
        }
    }

    /// Bind an IP-Root event id (called after ip_layer publishes kind 31900).
    pub fn bind_ip_root(&mut self, event_id: String) {
        self.network.ip_root_event = Some(event_id);
    }

    /// Bind a Vantage key.
    pub fn bind_vantage(&mut self, key: String) {
        self.economic.vantage_key = Some(key);
    }

    /// Bind an OSOVM/ZVM world endpoint.
    pub fn bind_world(&mut self, osovm: Option<String>, zvm: Option<String>) {
        self.world.osovm_endpoint = osovm;
        self.world.zvm_endpoint = zvm;
    }

    /// Add a web2 binding (OAuth handle, API endpoint, etc.)
    pub fn add_web2_binding(&mut self, kind: &str, handle: &str) {
        self.network.web2_bindings.push(Web2Binding {
            kind: kind.to_string(),
            handle: handle.to_string(),
        });
    }
}

// ─── Section structs ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct IdentitySection {
    pub agent_id: String,
    pub genesis_id: String,
    pub identity_commitment: String, // hex — harmonic signature
    pub capability_root: String,     // hex — derivation root
    pub birth_receipt_hash: String,
    pub owner: Option<String>, // owner agent_id if delegated
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct GenesisSection {
    pub bipon39_commitment: String,
    pub harmonic_signature: String,
    pub genesis_receipt_hash: String,
    pub sigil_hash: String,
    pub symbolic_address: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TemporalSection {
    pub born_at: u64, // Unix ms
    pub koodu_epoch: u64,
    pub koodu_cycle: u64,
    pub koodu_phase: u8,
    pub btc_height: Option<u64>,
    pub btc_anchor: Option<String>,
    pub gregorian_fallback: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SoulSection {
    pub primary_odu: u8,
    pub composed_odu: u16,
    pub temperament: String,
    pub orisha_alignment: String,
    pub destiny_threads: Vec<String>,
    pub hermetic_fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MemorySection {
    pub minipae_pubkey: String,
    pub memory_root: String,
    pub glyph_root: String, // GIX-FOLD-v1 glyph of genesis fact
    pub birth_glyph: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct NetworkSection {
    pub ip_root_event: Option<String>, // Nostr kind 31900
    pub nostr_pubkey: Option<String>,
    pub sui_address: Option<String>,
    pub btc_address: Option<String>,
    pub eth_address: Option<String>,
    pub mesh_id: Option<String>, // Meshtastic/Reticulum node id
    pub freenet_key: Option<String>,
    pub web2_bindings: Vec<Web2Binding>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Web2Binding {
    pub kind: String,   // "oauth", "api", "email", etc.
    pub handle: String, // the actual address/handle — never a secret
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct EconomicSection {
    pub sui_object_id: Option<String>, // on-chain NFT object id
    pub vantage_key: Option<String>,   // Vantage API key (public portion only)
    pub ase_balance: Option<f64>,
    pub reputation: f64,
    pub wallet_bindings: Vec<WalletBinding>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WalletBinding {
    pub chain: String,   // "sui", "btc", "eth", "nostr", etc.
    pub address: String, // public address only
    /// Static Poison Radar scan result at birth. None for chains that derive
    /// non-hex addresses (Nostr npub, minipae npub) where hex heuristics don't apply.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poison_scan: Option<crate::identity::poison_radar::PoisonReport>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct BodySection {
    pub device_binding: Option<DeviceRef>,
    pub twin_id: Option<String>,
    pub additional_devices: Vec<DeviceRef>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeviceRef {
    pub device_id: String,
    pub device_kind: String,
    pub bound_at: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct WorldSection {
    pub osovm_endpoint: Option<String>,
    pub zvm_endpoint: Option<String>,
    pub twin_root: Option<String>, // twin merkle root
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ProofSection {
    pub genesis_signature: String,
    pub witness_receipt: Option<String>,
    pub receipt_chain: Vec<String>, // ordered receipt hashes
    pub anchors: Vec<String>,       // Sui tx ids, Walrus blob ids, etc.
}
