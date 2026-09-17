//! Ọ̀ṢỌ́ WASM host environment interface.
//!
//! The generated WAT imports these functions from the "oso_host" module.
//! Real implementations are provided by the ỌSỌVM execution layer.
//! MockWasmHostEnv is used for testing and dry-run (oso-simulator integration).
//!
//! Phase 24.2 — WASM host interface.

use std::collections::BTreeMap;

pub trait WasmHostEnv {
    fn emit_receipt(&mut self, stage_id: i64, timestamp: i64);
    fn get_balance(&self, agent_idx: i32) -> i64;
    fn transfer(&mut self, from: i32, to: i32, amount: i64);
    fn get_timestamp(&self) -> i64;
}

pub struct MockWasmHostEnv {
    pub balances: BTreeMap<i32, i64>,
    pub receipts: Vec<(i64, i64)>,
    pub transfers: Vec<(i32, i32, i64)>,
    pub now: i64,
}

impl MockWasmHostEnv {
    pub fn new(now: i64) -> Self {
        Self {
            balances: BTreeMap::new(),
            receipts: Vec::new(),
            transfers: Vec::new(),
            now,
        }
    }

    pub fn fund(&mut self, agent_idx: i32, amount: i64) {
        *self.balances.entry(agent_idx).or_default() += amount;
    }
}

impl WasmHostEnv for MockWasmHostEnv {
    fn emit_receipt(&mut self, stage_id: i64, timestamp: i64) {
        self.receipts.push((stage_id, timestamp));
    }

    fn get_balance(&self, agent_idx: i32) -> i64 {
        *self.balances.get(&agent_idx).unwrap_or(&0)
    }

    fn transfer(&mut self, from: i32, to: i32, amount: i64) {
        let from_bal = self.balances.entry(from).or_default();
        *from_bal = (*from_bal - amount).max(0);
        *self.balances.entry(to).or_default() += amount;
        self.transfers.push((from, to, amount));
    }

    fn get_timestamp(&self) -> i64 {
        self.now
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_transfer_tracks_balances() {
        let mut env = MockWasmHostEnv::new(1_000_000);
        env.fund(1, 500);
        env.transfer(1, 2, 100);
        assert_eq!(env.get_balance(1), 400);
        assert_eq!(env.get_balance(2), 100);
        assert_eq!(env.transfers.len(), 1);
    }

    #[test]
    fn mock_receipt_records() {
        let mut env = MockWasmHostEnv::new(42);
        env.emit_receipt(3, 42);
        assert_eq!(env.receipts, vec![(3, 42)]);
    }
}
