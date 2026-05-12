#[cfg(test)]
mod receipt_tests {
    use omokoda_core::receipt::{Receipt, ReceiptStore};

    #[test]
    fn receipt_has_required_fields() {
        let r = Receipt::new("agent-001", "web_search", "bitcoin origin", "prev-hash");
        assert!(!r.agent_id.is_empty());
        assert!(!r.action.is_empty());
        assert!(!r.payload.is_empty());
        assert!(!r.receipt_id.is_empty());
        assert_eq!(r.previous_hash, "prev-hash");
        assert!(r.timestamp > 0);
    }

    #[test]
    fn same_input_different_receipts() {
        // Each receipt gets a unique ID even with same inputs
        let a = Receipt::new("agent-001", "web_search", "query", "hash");
        let b = Receipt::new("agent-001", "web_search", "query", "hash");
        assert_ne!(a.receipt_id, b.receipt_id);
    }

    #[test]
    fn receipt_chain_verification() {
        let mut store = ReceiptStore::new();
        let r1 = Receipt::new("agent-001", "act1", "p1", store.last_hash());
        let id1 = r1.receipt_id.clone();
        store.record(r1);

        let r2 = Receipt::new("agent-001", "act2", "p2", store.last_hash());
        let id2 = r2.receipt_id.clone();
        store.record(r2);

        assert_eq!(id1, store.get(&id1).unwrap().receipt_id);
        assert_eq!(id2, store.get(&id2).unwrap().receipt_id);
        assert_eq!(store.get(&id2).unwrap().previous_hash, id1);
        assert!(store.verify_chain());
    }

    #[test]
    fn tampered_chain_fails_verification() {
        let mut store = ReceiptStore::new();
        let r1 = Receipt::new("agent-001", "act1", "p1", store.last_hash());
        store.record(r1);

        // Manually record a receipt with wrong previous hash
        let r2 = Receipt::new("agent-001", "act2", "p2", "WRONG_HASH");
        store.record(r2);

        assert!(!store.verify_chain());
    }
}
