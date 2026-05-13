use omokoda_core::identity::AgentId;
use omokoda_core::receipt::{Receipt, ReceiptStore};

#[cfg(test)]
mod receipt_tests {
    use super::*;
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;
    use rand::RngCore;

    fn generate_key() -> SigningKey {
        let mut entropy = [0u8; 32];
        OsRng.fill_bytes(&mut entropy);
        SigningKey::from_bytes(&entropy)
    }

    #[test]
    fn new_merkle_receipt_creation() {
        let key = generate_key();
        let agent_id = AgentId::from_str("agent-001");
        
        let r = Receipt::new_merkle(&agent_id, "web_search", "bitcoin origin", "prev-hash", "merkle-root", &key);
        assert!(!r.agent_id.as_str().is_empty());
        assert_eq!(r.action, "web_search");
        assert!(!r.receipt_id.is_empty());
        assert!(!r.signature.is_empty());
    }

    #[test]
    fn receipt_determinism() {
        let agent_id = AgentId::from_str("agent-001");
        
        // Nonce and timestamp make it non-deterministic by default in new_merkle,
        // but calculate_id itself is deterministic.
        let id1 = Receipt::calculate_id(&agent_id, "act", "pay", "prev", "root", 100, 123);
        let id2 = Receipt::calculate_id(&agent_id, "act", "pay", "prev", "root", 100, 123);
        assert_eq!(id1, id2);
    }

    #[test]
    fn signature_verification() {
        let key = generate_key();
        let public_key = key.verifying_key().to_bytes();
        let agent_id = AgentId::from_str("agent-001");

        let a = Receipt::new_merkle(&agent_id, "web_search", "query", "hash", "root", &key);
        let b = Receipt::new_merkle(&agent_id, "web_search", "query", "hash", "root", &key);
        
        assert!(a.verify(&public_key).is_ok());
        assert!(b.verify(&public_key).is_ok());
        assert_ne!(a.receipt_id, b.receipt_id); // Different nonces
    }

    #[test]
    fn detection_of_tampering() {
        let key = generate_key();
        let public_key = key.verifying_key().to_bytes();
        let agent_id = AgentId::from_str("agent-001");

        let r_orig = Receipt::new_merkle(&agent_id, "act", "p", "prev", "root", &key);
        let mut r = r_orig.clone();
        
        assert!(r.verify(&public_key).is_ok());

        // Tamper with action
        r.action = "malicious".to_string();
        assert!(r.verify(&public_key).is_err());

        // Tamper with agent_id
        r = r_orig.clone();
        r.agent_id = AgentId::from_str("agent-666");
        assert!(r.verify(&public_key).is_err());

        // Tamper with signature
        r = r_orig.clone();
        r.signature = "0".repeat(128);
        assert!(r.verify(&public_key).is_err());
    }

    #[test]
    fn receipt_store_chaining() {
        let key = generate_key();
        let mut store = ReceiptStore::new();
        let agent_id = AgentId::from_str("agent-001");

        assert_eq!(store.count(), 0);
        assert_eq!(store.last_hash(), "0".repeat(64));

        let r1 = Receipt::new_merkle(&agent_id, "act1", "p1", store.last_hash(), &store.current_merkle_root(), &key);
        store.record(r1.clone());
        assert_eq!(store.count(), 1);
        assert_eq!(store.last_hash(), r1.receipt_id);

        let r2 = Receipt::new_merkle(&agent_id, "act2", "p2", store.last_hash(), &store.current_merkle_root(), &key);
        store.record(r2.clone());
        assert_eq!(store.count(), 2);
        assert_eq!(store.last_hash(), r2.receipt_id);

        assert!(store.verify_chain());
    }

    #[test]
    fn merkle_tree_root_evolution() {
        let key = generate_key();
        let mut store = ReceiptStore::new();
        let agent_id = AgentId::from_str("agent-001");

        let root0 = store.current_merkle_root();
        assert_eq!(root0, "0".repeat(64));

        let r1 = Receipt::new_merkle(&agent_id, "act1", "p1", store.last_hash(), &root0, &key);
        store.record(r1.clone());
        let root1 = store.current_merkle_root();
        assert_ne!(root1, root0);
        assert_eq!(root1, r1.receipt_id); // Single leaf = root

        let r2 = Receipt::new_merkle(&agent_id, "act2", "p2", store.last_hash(), &root1, &key);
        store.record(r2.clone());
        let root2 = store.current_merkle_root();
        assert_ne!(root2, root1);
        assert_ne!(root2, r2.receipt_id);
    }
}
