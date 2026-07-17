use crate::identity::AgentId;
use omokoda_hermetic::plane::Plane;
use serde::{Deserialize, Serialize};

/// Proof of Cognitive Work — unfakeable BB-grounded computational proof.
/// Justice uses this as a hard floor for act tier elevation above ACT_TIER_1.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoCWProof {
    /// Number of Turing Machine steps actually executed (≥ BB(n) for claimed tier)
    pub steps: u64,
    /// BB bound claimed (1, 6, 21, 107, or 47_176_870)
    pub bb_bound: u64,
    /// SHA3-256 hash of the TM tape at halt — verifiable by Justice without re-running
    pub tape_hash: String,
}

impl PoCWProof {
    /// Returns true if steps meet the minimum for the claimed BB bound.
    pub fn is_valid(&self) -> bool {
        self.steps >= self.bb_bound && !self.tape_hash.is_empty()
    }

    /// Minimum steps required for act tier elevation thresholds.
    pub fn min_for_tier(tier: u8) -> u64 {
        match tier {
            0 => 0,
            1 => 21,             // BB(3)
            2 => 107,            // BB(4)
            3 | 4 => 47_176_870, // BB(5)
            _ => 47_176_870,
        }
    }
}

/// EpistemicSeverity: how much the multi-model wisdom ensemble disagreed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EpistemicSeverity {
    Unanimous,
    Strong,
    Moderate,
    Severe,
}

/// Immutable action receipt. `dry_run` is structurally false — always real execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActReceipt {
    pub agent_id: AgentId,
    pub action: String,
    pub payload: String,
    pub timestamp: u64,
    pub receipt_id: String,
    /// dry_run is STRUCTURALLY false. Never set to true. Receipts are real state transitions.
    pub dry_run: bool,
    /// Optional proof of cognitive work for act tier elevation.
    pub proof_of_work: Option<PoCWProof>,
    /// Epistemic severity from wisdom ensemble (None if single-model).
    pub epistemic_severity: Option<EpistemicSeverity>,
    /// BLAKE3 hash of the previous receipt for chain integrity.
    pub previous_hash: Option<String>,
    /// The sim-to-real verification fidelity this action actually reached
    /// (see omokoda_hermetic::plane). Defaults to Physical -- an unverified
    /// claim -- in `new()`; call `with_plane` only after a real
    /// `plane::verify_plane` call returned `Verified`, never to assert a
    /// plane the action wasn't actually checked against.
    pub plane: Plane,
}

impl ActReceipt {
    pub fn new(agent_id: AgentId, action: String, payload: String, timestamp: u64) -> Self {
        let receipt_id = {
            let mut hasher = blake3::Hasher::new();
            hasher.update(agent_id.as_str().as_bytes());
            hasher.update(action.as_bytes());
            hasher.update(payload.as_bytes());
            hasher.update(&timestamp.to_le_bytes());
            hasher.finalize().to_hex().to_string()
        };
        Self {
            agent_id,
            action,
            payload,
            timestamp,
            receipt_id,
            dry_run: false,
            proof_of_work: None,
            epistemic_severity: None,
            previous_hash: None,
            plane: Plane::Physical,
        }
    }

    pub fn with_pocw(mut self, proof: PoCWProof) -> Self {
        self.proof_of_work = Some(proof);
        self
    }

    pub fn with_previous_hash(mut self, prev: String) -> Self {
        self.previous_hash = Some(prev);
        self
    }

    /// Attach a real, already-verified plane. Callers must have obtained
    /// `PlaneVerification::Verified` from `omokoda_hermetic::plane::verify_plane`
    /// first -- this setter does not itself verify anything.
    pub fn with_plane(mut self, plane: Plane) -> Self {
        self.plane = plane;
        self
    }

    /// Returns true if this receipt has valid PoCW for the given act tier.
    pub fn meets_pocw_floor(&self, tier: u8) -> bool {
        if tier == 0 {
            return true;
        }
        match &self.proof_of_work {
            None => false,
            Some(proof) => proof.steps >= PoCWProof::min_for_tier(tier) && proof.is_valid(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::AgentId;

    #[test]
    fn dry_run_is_always_false() {
        let r = ActReceipt::new(
            AgentId::from_str("agent1"),
            "think".to_string(),
            "hash123".to_string(),
            42,
        );
        assert!(
            !r.dry_run,
            "dry_run must always be false — structural invariant"
        );
    }

    #[test]
    fn pocw_validates_step_count() {
        let valid = PoCWProof {
            steps: 21,
            bb_bound: 21,
            tape_hash: "abc".to_string(),
        };
        assert!(valid.is_valid());
        let invalid = PoCWProof {
            steps: 20,
            bb_bound: 21,
            tape_hash: "abc".to_string(),
        };
        assert!(!invalid.is_valid());
    }

    #[test]
    fn tier_1_floor_is_bb3() {
        assert_eq!(PoCWProof::min_for_tier(1), 21);
    }

    #[test]
    fn tier_3_floor_is_bb5() {
        assert_eq!(PoCWProof::min_for_tier(3), 47_176_870);
    }

    #[test]
    fn receipt_meets_pocw_floor_for_tier_zero_always() {
        let r = ActReceipt::new(
            AgentId::from_str("a"),
            "act".to_string(),
            "p".to_string(),
            0,
        );
        assert!(r.meets_pocw_floor(0));
    }

    #[test]
    fn receipt_fails_pocw_floor_tier_1_without_proof() {
        let r = ActReceipt::new(
            AgentId::from_str("a"),
            "act".to_string(),
            "p".to_string(),
            0,
        );
        assert!(!r.meets_pocw_floor(1));
    }

    #[test]
    fn receipt_meets_pocw_floor_with_valid_proof() {
        let proof = PoCWProof {
            steps: 107,
            bb_bound: 107,
            tape_hash: "abc".to_string(),
        };
        let r = ActReceipt::new(
            AgentId::from_str("a"),
            "act".to_string(),
            "p".to_string(),
            0,
        )
        .with_pocw(proof);
        assert!(r.meets_pocw_floor(2));
    }

    #[test]
    fn receipt_chain_links_via_previous_hash() {
        let r1 = ActReceipt::new(
            AgentId::from_str("a"),
            "act".to_string(),
            "p1".to_string(),
            1,
        );
        let r2 = ActReceipt::new(
            AgentId::from_str("a"),
            "act".to_string(),
            "p2".to_string(),
            2,
        )
        .with_previous_hash(r1.receipt_id.clone());
        assert_eq!(r2.previous_hash, Some(r1.receipt_id));
    }
}
