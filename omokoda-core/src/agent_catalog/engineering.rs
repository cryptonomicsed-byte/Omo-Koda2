use super::AgentRole;

pub fn roles() -> Vec<AgentRole> {
    vec![
        AgentRole {
            id: "embedded-firmware-engineer".to_string(),
            title: "Embedded Firmware Engineer".to_string(),
            division: "engineering".to_string(),
            context: "Low-level hardware firmware for sovereign witness nodes".to_string(),
            responsibilities: vec![
                "Write deterministic embedded Rust/C firmware".to_string(),
                "Implement hardware attestation protocols".to_string(),
                "Ensure reproducible builds and cryptographic proofs".to_string(),
            ],
            capabilities: vec![
                "embedded-rust".to_string(),
                "hardware-attestation".to_string(),
                "deterministic-builds".to_string(),
            ],
            output_format: "code + documentation".to_string(),
        },
        AgentRole {
            id: "autonomous-optimization-architect".to_string(),
            title: "Autonomous Optimization Architect".to_string(),
            division: "engineering".to_string(),
            context: "Self-improving agent systems and resource optimization".to_string(),
            responsibilities: vec![
                "Design feedback loops for continuous improvement".to_string(),
                "Optimize agent resource usage and scheduling".to_string(),
                "Propose and gate self-modification patches".to_string(),
            ],
            capabilities: vec![
                "system-design".to_string(),
                "optimization".to_string(),
                "skill-patch".to_string(),
            ],
            output_format: "architecture docs + proposals".to_string(),
        },
        AgentRole {
            id: "backend-architect".to_string(),
            title: "Backend Architect".to_string(),
            division: "engineering".to_string(),
            context: "Sovereign backend services, APIs, and inter-service protocols".to_string(),
            responsibilities: vec![
                "Design scalable backend architecture".to_string(),
                "Define inter-service protocols (DIP, VCP, ARP)".to_string(),
                "Ensure security and receipt-based accountability".to_string(),
            ],
            capabilities: vec![
                "rust".to_string(),
                "api-design".to_string(),
                "protocol-design".to_string(),
            ],
            output_format: "design docs + code".to_string(),
        },
    ]
}
