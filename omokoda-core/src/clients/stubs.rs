//! Stub implementations of all 5 external client traits.
//! Used as fallback when external services (Bipon39-Rust-, vanity2,
//! Ritual-codex-Julia, ifascript, Nex-) are not reachable.
//! Logic is preserved from the original identity/ modules so existing
//! tests continue to pass.

use super::*;

// ─── StubBiponClient ──────────────────────────────────────────────────────────

pub struct StubBiponClient;

impl BiponClient for StubBiponClient {
    fn entropy_to_mnemonic(&self, entropy: &[u8]) -> Result<MnemonicResult, ClientError> {
        // Delegate to vendored bipon39-stub crate
        let words = bipon39::entropy_to_mnemonic(entropy)
            .map_err(|e| ClientError::InvalidInput(e.to_string()))?;
        let phrase = words.join(" ");
        let word_count = words.len() as u8;
        Ok(MnemonicResult { phrase, word_count })
    }

    fn mnemonic_to_seed(&self, phrase: &str, passphrase: &str) -> Result<[u8; 64], ClientError> {
        let words: Vec<&str> = phrase.split_whitespace().collect();
        let seed = bipon39::mnemonic_to_seed(&words, passphrase)
            .map_err(|e| ClientError::InvalidInput(e.to_string()))?;
        let mut out = [0u8; 64];
        let len = seed.len().min(64);
        out[..len].copy_from_slice(&seed[..len]);
        Ok(out)
    }

    fn personality_profile(&self, _mnemonic: &str) -> Result<PersonalityResult, ClientError> {
        // Stub returns a balanced distribution across 7 principles
        Ok(PersonalityResult {
            distribution: [14, 14, 14, 14, 15, 15, 14],
            elemental: [0.2, 0.2, 0.2, 0.2, 0.2],
            dominant: 0,
        })
    }
}

// ─── StubVanityClient ─────────────────────────────────────────────────────────

pub struct StubVanityClient;

impl VanityClient for StubVanityClient {
    fn derive_wallet(&self, mnemonic: &str, passphrase: &str) -> Result<WalletResult, ClientError> {
        use hmac::{Hmac, Mac};
        use sha2::Sha512;

        let salt = format!("mnemonic{}", passphrase);
        let mut seed = [0u8; 64];
        pbkdf2::pbkdf2::<Hmac<Sha512>>(mnemonic.as_bytes(), salt.as_bytes(), 2048, &mut seed[..])
            .map_err(|e| ClientError::InvalidInput(e.to_string()))?;

        // SLIP-0010 master key
        let mut hmac = Hmac::<Sha512>::new_from_slice(b"ed25519 seed")
            .map_err(|e| ClientError::InvalidInput(e.to_string()))?;
        hmac.update(&seed);
        let result = hmac.finalize().into_bytes();

        let mut signing_key = [0u8; 32];
        signing_key.copy_from_slice(&result[..32]);

        // Stub address: hex of first 20 bytes prefixed with 0x
        let address = format!("0x{}", hex::encode(&signing_key[..20]));

        Ok(WalletResult { signing_key, address })
    }

    fn cloak_display(&self, words: &[String], offset: u8) -> Result<CloakResult, ClientError> {
        let cloaked_words = words
            .iter()
            .enumerate()
            .map(|(i, w)| {
                let shift = ((i as u8).wrapping_add(offset)) % 26;
                w.chars()
                    .map(|c| {
                        if c.is_ascii_lowercase() {
                            (b'a' + (c as u8 - b'a' + shift) % 26) as char
                        } else if c.is_ascii_uppercase() {
                            (b'A' + (c as u8 - b'A' + shift) % 26) as char
                        } else {
                            c
                        }
                    })
                    .collect()
            })
            .collect();
        Ok(CloakResult { cloaked_words })
    }

    fn scan_poison(&self, candidate: &str, known: &[String]) -> Result<PoisonScanResult, ClientError> {
        for k in known {
            if candidate.len() >= 4 && k.len() >= 4 {
                let prefix_match = &candidate[..4] == &k[..4];
                let suffix_match = &candidate[candidate.len() - 4..] == &k[k.len() - 4..];
                if (prefix_match || suffix_match) && candidate != k.as_str() {
                    return Ok(PoisonScanResult { is_safe: false, similar_to: Some(k.clone()) });
                }
            }
        }
        Ok(PoisonScanResult { is_safe: true, similar_to: None })
    }
}

// ─── StubRitualClient ─────────────────────────────────────────────────────────

pub struct StubRitualClient;

/// BB known values: BB(n) = max steps a halting n-state TM can make
const BB_KNOWN: [u64; 6] = [0, 1, 6, 21, 107, 47_176_870];

impl RitualClient for StubRitualClient {
    fn verify_pocw(&self, tier: u8, steps: u64) -> Result<PocwResult, ClientError> {
        let idx = (tier as usize).min(5);
        let floor = BB_KNOWN[idx];
        Ok(PocwResult { verified: steps >= floor, floor })
    }

    fn score_bbu(&self, code: &str) -> Result<BbuResult, ClientError> {
        // Heuristic: entropy estimate from byte distribution
        let len = code.len() as f64;
        if len == 0.0 {
            return Ok(BbuResult { score: 1.0 });
        }
        let mut counts = [0u32; 256];
        for b in code.bytes() {
            counts[b as usize] += 1;
        }
        let entropy: f64 = counts.iter()
            .filter(|&&c| c > 0)
            .map(|&c| {
                let p = c as f64 / len;
                -p * p.log2()
            })
            .sum();
        let score = 1.0 + (entropy / 8.0 * 40.0);
        Ok(BbuResult { score: score.min(47.1) })
    }

    fn augury_predict(&self, patterns: &[MemoryPattern]) -> Result<AuguryResult, ClientError> {
        // Stub: return the most recently accessed branch
        let best = patterns.iter()
            .max_by(|a, b| a.timestamp.partial_cmp(&b.timestamp).unwrap_or(std::cmp::Ordering::Equal));
        match best {
            Some(p) => Ok(AuguryResult { predicted_branch: p.branch_id.clone(), confidence: 0.6 }),
            None => Ok(AuguryResult { predicted_branch: "root".to_string(), confidence: 0.3 }),
        }
    }
}

// ─── StubIfascriptClient ─────────────────────────────────────────────────────

pub struct StubIfascriptClient;

impl IfascriptClient for StubIfascriptClient {
    fn lookup_odu(&self, index: u8) -> Result<OduResult, ClientError> {
        // Delegate to vendored ifascript-stub
        let name = ifascript::odu::ODU_TABLE
            .get(index as usize)
            .map(|(n, _)| n.to_string())
            .unwrap_or_else(|| format!("Odu-{}", index));
        Ok(OduResult {
            id: index,
            name,
            prescription: ifascript::ebo::cast(index).to_string(),
        })
    }

    fn cast_ebo(&self, odu: u8) -> Result<EboResult, ClientError> {
        let message = ifascript::ebo::cast(odu).to_string();
        // Heuristic: index range → severity
        let level = match odu {
            0..=85 => EboLevel::Advisory,
            86..=170 => EboLevel::Caution,
            _ => EboLevel::Critical,
        };
        Ok(EboResult { level, message })
    }

    fn generate_entropy(&self, seed: &[u8]) -> Result<Vec<u8>, ClientError> {
        Ok(ifascript::entropy::generate(seed))
    }

    fn larql_query(&self, _query: &str, _tier: u8) -> Result<LarqlResult, ClientError> {
        // Stub: LARQL lives in ifascript — return empty result until real service available
        Ok(LarqlResult {
            steps: vec!["[stub] LARQL query — connect ifascript service for results".to_string()],
            confidence: 0.0,
            human_override: true,
        })
    }
}

// ─── StubNexClient ────────────────────────────────────────────────────────────

pub struct StubNexClient;

impl NexClient for StubNexClient {
    fn submit_graph(&self, nodes: Vec<GraphNode>) -> Result<GraphResult, ClientError> {
        // Stub: acknowledge graph submission without distributed execution
        let graph_id = format!("stub-graph-{}", nodes.len());
        Ok(GraphResult { graph_id, nodes_executed: nodes.len() as u32 })
    }

    fn graph_status(&self, graph_id: &str) -> Result<GraphStatus, ClientError> {
        Ok(GraphStatus {
            graph_id: graph_id.to_string(),
            state: GraphState::Complete,
        })
    }
}
