use super::AgentRole;

pub fn roles() -> Vec<AgentRole> {
    vec![
        AgentRole {
            id: "simulation-runner".to_string(),
            title: "Simulation Runner".to_string(),
            division: "osovm".to_string(),
            context: "Runs deterministic trajectory simulations in OSOVM/Julia".to_string(),
            responsibilities: vec![
                "Execute 100k-trajectory ScarabSwarm simulations".to_string(),
                "Produce ProofOfSimulation receipts".to_string(),
                "Validate simulation determinism across nodes".to_string(),
            ],
            capabilities: vec![
                "julia".to_string(),
                "mujoco".to_string(),
                "proof-of-simulation".to_string(),
            ],
            output_format: "SimReceipt JSON".to_string(),
        },
        AgentRole {
            id: "witness-attestor".to_string(),
            title: "Witness Attestor".to_string(),
            division: "osovm".to_string(),
            context: "Hardware attestation for sovereign witness nodes".to_string(),
            responsibilities: vec![
                "Generate WitnessAttestation (Nostr kind 31020)".to_string(),
                "Sign ObservationBundles with hardware keys".to_string(),
                "Verify firmware manifests".to_string(),
            ],
            capabilities: vec![
                "ed25519".to_string(),
                "nostr".to_string(),
                "hardware-attestation".to_string(),
            ],
            output_format: "Nostr event kind:31020".to_string(),
        },
        AgentRole {
            id: "compute-proof-verifier".to_string(),
            title: "Compute Proof Verifier".to_string(),
            division: "osovm".to_string(),
            context: "Verifies proofs of computation from OSOVM nodes".to_string(),
            responsibilities: vec![
                "Verify ProofOfSimulation receipts from peers".to_string(),
                "Cross-check simulation outputs for determinism".to_string(),
                "Emit verification receipts for the ASE chain".to_string(),
            ],
            capabilities: vec![
                "proof-verification".to_string(),
                "arp-receipts".to_string(),
                "consensus".to_string(),
            ],
            output_format: "ARP receipt JSON".to_string(),
        },
    ]
}
