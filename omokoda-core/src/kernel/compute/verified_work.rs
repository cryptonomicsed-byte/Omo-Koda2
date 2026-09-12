use super::attestation::HardwareAttestation;
use super::telemetry::GpuTelemetry;

/// The atomic compute unit in the Omo-Koda2 / OSOVM economy.
///
/// Represents the complete chain:
///   GPU contribution → hardware identity → attestation → lease
///   → actual GPU-seconds → telemetry → Zàngbétò receipt → OSOVM verification
///   → Dopamine credit
///
/// NOT created by self-report. Created only after full chain is satisfied.
#[derive(Debug, Clone)]
pub struct VerifiedGPUWork {
    pub work_id:              String,
    pub contributor_id:       String,  // agent_id
    pub device_id:            String,  // /devices/gpu/0001
    pub gpu_model:            String,

    // Lease provenance
    pub lease_id:             String,
    pub start_time:           u64,
    pub end_time:             u64,
    pub gpu_seconds:          f64,

    // Workload identity
    pub workload_hash:        String,
    pub workload_type:        WorkloadKind,
    pub utilization:          f32,     // avg over the lease

    // Commitments — verified before Dopamine is issued
    pub output_commitment:    String,  // SHA-256 of workload output
    pub telemetry_commitment: String,  // from GpuTelemetry.commitment_hash

    // Proof chain
    pub witness_receipts:     Vec<String>,   // receipt_ids from independent witnesses
    pub hardware_attestation: HardwareAttestation,
    pub osovm_proof:          Option<String>, // filled after OSOVM verification

    // Economic output
    pub compute_score:        Option<f64>,
    pub dopamine_allocation:  Option<u64>,   // micro-Dopamine units

    // Zàngbétò anchor
    pub zangbeto_receipt_id:  Option<String>,
    pub created_at:           u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkloadKind {
    Training,
    Inference,
    Simulation,
    Rendering,
    ZkProof,
    Encoding,
    Custom(String),
}

impl std::fmt::Display for WorkloadKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Training   => write!(f, "training"),
            Self::Inference  => write!(f, "inference"),
            Self::Simulation => write!(f, "simulation"),
            Self::Rendering  => write!(f, "rendering"),
            Self::ZkProof    => write!(f, "zk_proof"),
            Self::Encoding   => write!(f, "encoding"),
            Self::Custom(s)  => write!(f, "{s}"),
        }
    }
}

impl VerifiedGPUWork {
    pub fn new(
        contributor_id: impl Into<String>,
        device_id: impl Into<String>,
        gpu_model: impl Into<String>,
        lease_id: impl Into<String>,
        start_time: u64,
        end_time: u64,
        gpu_seconds: f64,
        workload_hash: impl Into<String>,
        workload_type: WorkloadKind,
        telemetry: &GpuTelemetry,
        attestation: HardwareAttestation,
        output_commitment: impl Into<String>,
    ) -> Self {
        Self {
            work_id:              uuid_v4(),
            contributor_id:       contributor_id.into(),
            device_id:            device_id.into(),
            gpu_model:            gpu_model.into(),
            lease_id:             lease_id.into(),
            start_time,
            end_time,
            gpu_seconds,
            workload_hash:        workload_hash.into(),
            workload_type,
            utilization:          telemetry.avg_utilization,
            output_commitment:    output_commitment.into(),
            telemetry_commitment: telemetry.commitment_hash.clone(),
            witness_receipts:     vec![],
            hardware_attestation: attestation,
            osovm_proof:          None,
            compute_score:        None,
            dopamine_allocation:  None,
            zangbeto_receipt_id:  None,
            created_at:           now_secs(),
        }
    }

    pub fn add_witness(&mut self, receipt_id: impl Into<String>) {
        self.witness_receipts.push(receipt_id.into());
    }

    pub fn set_osovm_proof(&mut self, proof: impl Into<String>) {
        self.osovm_proof = Some(proof.into());
    }

    pub fn set_dopamine(&mut self, score: f64, allocation: u64, zangbeto_id: impl Into<String>) {
        self.compute_score       = Some(score);
        self.dopamine_allocation = Some(allocation);
        self.zangbeto_receipt_id = Some(zangbeto_id.into());
    }

    /// True only when the full proof chain is satisfied (OSOVM + witnesses + Zàngbétò)
    pub fn is_fully_verified(&self) -> bool {
        self.osovm_proof.is_some()
            && !self.witness_receipts.is_empty()
            && self.zangbeto_receipt_id.is_some()
            && self.hardware_attestation.verified
    }
}

fn uuid_v4() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::SystemTime;
    let mut h = DefaultHasher::new();
    SystemTime::now().hash(&mut h);
    format!("{:016x}{:016x}", h.finish(), h.finish().wrapping_mul(0xdead_beef))
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
