use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Receipt {
    pub agent_id: String,
    pub action: String,
    pub payload: String,
    pub receipt_id: String,
    pub previous_hash: String,
    pub timestamp: u64,
    pub dry_run: bool,
}

impl Receipt {
    pub fn new(agent_id: &str, action: &str, params: &str, previous_hash: &str) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before unix epoch")
            .as_secs();
        let timestamp_nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before unix epoch")
            .as_nanos();

        let payload = blake3_hash_hex(&[action.as_bytes(), params.as_bytes()]);
        let receipt_id = blake3_hash_hex(&[
            agent_id.as_bytes(),
            action.as_bytes(),
            params.as_bytes(),
            previous_hash.as_bytes(),
            timestamp_nanos.to_string().as_bytes(),
        ]);

        Self {
            agent_id: agent_id.to_string(),
            action: action.to_string(),
            payload,
            receipt_id,
            previous_hash: previous_hash.to_string(),
            timestamp,
            dry_run: false,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReceiptStore {
    receipts: HashMap<String, Receipt>,
    last_hash: String,
    chain: Vec<String>,
}

impl ReceiptStore {
    pub fn new() -> Self {
        Self {
            receipts: HashMap::new(),
            last_hash: "0".repeat(64),
            chain: Vec::new(),
        }
    }

    pub fn record(&mut self, receipt: Receipt) {
        let id = receipt.receipt_id.clone();
        self.last_hash = id.clone();
        self.chain.push(id.clone());
        self.receipts.insert(id, receipt);
    }

    pub fn get(&self, receipt_id: &str) -> Option<&Receipt> {
        self.receipts.get(receipt_id)
    }

    pub fn last_hash(&self) -> &str {
        &self.last_hash
    }

    pub fn count(&self) -> usize {
        self.receipts.len()
    }

    pub fn verify_chain(&self) -> bool {
        let mut current_expected_prev = "0".repeat(64);
        for id in &self.chain {
            if let Some(r) = self.receipts.get(id) {
                if r.previous_hash != current_expected_prev {
                    return false;
                }
                current_expected_prev = r.receipt_id.clone();
            } else {
                return false;
            }
        }
        true
    }
}

fn blake3_hash_hex(parts: &[&[u8]]) -> String {
    let mut hasher = blake3::Hasher::new();
    for part in parts {
        hasher.update(part);
    }
    hasher.finalize().to_hex().to_string()
}
