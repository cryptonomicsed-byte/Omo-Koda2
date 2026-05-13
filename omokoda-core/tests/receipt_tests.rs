#[cfg(test)]
mod receipt_tests {
    use omokoda_core::receipt::{Receipt, ReceiptStore};
    use ed25519_dalek::SigningKey;
    use rand::Rng;

    fn mock_signing_key() -> SigningKey {
        let mut seed = [0u8; 32];
        rand::thread_rng().fill(&mut seed);
        SigningKey::from_bytes(&seed)
    }

    #[test]
    fn receipt_has_required_fields() {
        let key = mock_signing_key();
        let r = Receipt::new_merkle("agent-001", "web_search", "bitcoin origin", "prev-hash", "merkle-root", &key);
        assert!(!r.agent_id.is_empty());
        assert!(!r.action.is_empty());
        assert!(!r.payload.is_empty());
        assert!(!r.receipt_id.is_empty());
        assert_eq!(r.previous_hash, "prev-hash");
        assert_eq!(r.merkle_root, "merkle-root");
        assert!(!r.signature.is_empty());
        assert!(r.timestamp > 0);
    }

    #[test]
    fn same_input_different_receipts() {
        let key = mock_signing_key();
        let a = Receipt::new_merkle("agent-001", "web_search", "query", "hash", "root", &key);
        let b = Receipt::new_merkle("agent-001", "web_search", "query", "hash", "root", &key);
        assert_ne!(a.receipt_id, b.receipt_id);
    }

    #[test]
    fn receipt_tamper_detection() {
        let key = mock_signing_key();
        let pub_key = key.verifying_key().to_bytes();
        let r_orig = Receipt::new_merkle("agent-001", "act", "p", "prev", "root", &key);
        
        assert!(r_orig.verify(&pub_key).is_ok());

        // Tamper with action
        let mut r = r_orig.clone();
        r.action = "tampered".to_string();
        assert!(r.verify(&pub_key).is_err());

        // Tamper with agent_id
        let mut r = r_orig.clone();
        r.agent_id = "agent-666".to_string();
        assert!(r.verify(&pub_key).is_err());

        // Tamper with payload
        let mut r = r_orig.clone();
        r.payload = "fake-payload".to_string();
        assert!(r.verify(&pub_key).is_err());

        // Tamper with previous_hash
        let mut r = r_orig.clone();
        r.previous_hash = "fake-prev".to_string();
        assert!(r.verify(&pub_key).is_err());

        // Tamper with merkle_root
        let mut r = r_orig.clone();
        r.merkle_root = "fake-root".to_string();
        assert!(r.verify(&pub_key).is_err());

        // Tamper with timestamp
        let mut r = r_orig.clone();
        r.timestamp += 1;
        assert!(r.verify(&pub_key).is_err());
    }

    #[test]
    fn receipt_signature_verification() {
        let key = mock_signing_key();
        let pub_key = key.verifying_key().to_bytes();
        let r = Receipt::new_merkle("agent-001", "act", "p", "prev", "root", &key);
        
        assert!(r.verify(&pub_key).is_ok());

        // Wrong public key
        let wrong_key = mock_signing_key().verifying_key().to_bytes();
        assert!(r.verify(&wrong_key).is_err());
    }

    #[test]
    fn receipt_chain_verification() {
        let mut store = ReceiptStore::new();
        let key = mock_signing_key();
        
        let r1 = Receipt::new_merkle("agent-001", "act1", "p1", store.last_hash(), &store.current_merkle_root(), &key);
        let id1 = r1.receipt_id.clone();
        store.record(r1);

        let r2 = Receipt::new_merkle("agent-001", "act2", "p2", store.last_hash(), &store.current_merkle_root(), &key);
        let id2 = r2.receipt_id.clone();
        store.record(r2);

        assert_eq!(id1, store.get(&id1).unwrap().receipt_id);
        assert_eq!(id2, store.get(&id2).unwrap().receipt_id);
        assert_eq!(store.get(&id2).unwrap().previous_hash, id1);
        assert!(store.verify_chain());
    }

    #[test]
    fn merkle_root_changes_on_insert() {
        let mut store = ReceiptStore::new();
        let key = mock_signing_key();
        
        let root0 = store.current_merkle_root();
        
        let r1 = Receipt::new_merkle("agent-001", "act1", "p1", store.last_hash(), &root0, &key);
        store.record(r1);
        let root1 = store.current_merkle_root();
        assert_ne!(root0, root1);

        let r2 = Receipt::new_merkle("agent-001", "act2", "p2", store.last_hash(), &root1, &key);
        store.record(r2);
        let root2 = store.current_merkle_root();
        assert_ne!(root1, root2);
    }
}
