use super::AgentRole;

pub fn roles() -> Vec<AgentRole> {
    vec![
        AgentRole {
            id: "node-operator".to_string(),
            title: "Node Operator".to_string(),
            division: "sovereign".to_string(),
            context: "Operates and maintains a sovereign node in the ecosystem".to_string(),
            responsibilities: vec![
                "Keep node healthy and fully synced".to_string(),
                "Manage stake and emission claims".to_string(),
                "Monitor mesh connectivity and uptime".to_string(),
            ],
            capabilities: vec![
                "node-management".to_string(),
                "stake-management".to_string(),
                "monitoring".to_string(),
            ],
            output_format: "status reports + ARP receipts".to_string(),
        },
        AgentRole {
            id: "receipt-auditor".to_string(),
            title: "Receipt Auditor".to_string(),
            division: "sovereign".to_string(),
            context: "Audits ARP receipts for the Zangbeto security layer".to_string(),
            responsibilities: vec![
                "Verify ARP receipt chains for tamper evidence".to_string(),
                "Flag anomalous receipt patterns".to_string(),
                "Report findings to governance council".to_string(),
            ],
            capabilities: vec![
                "arp-audit".to_string(),
                "receipt-chain-verify".to_string(),
                "governance".to_string(),
            ],
            output_format: "audit report JSON".to_string(),
        },
        AgentRole {
            id: "mesh-router".to_string(),
            title: "Mesh Router".to_string(),
            division: "sovereign".to_string(),
            context: "Routes messages across the sovereign mesh network".to_string(),
            responsibilities: vec![
                "Maintain DIP routing tables".to_string(),
                "Forward messages across Nostr/Meshtastic/libp2p adapters".to_string(),
                "Ensure offline continuity for disconnected nodes".to_string(),
            ],
            capabilities: vec![
                "dip".to_string(),
                "nostr".to_string(),
                "meshtastic".to_string(),
                "libp2p".to_string(),
            ],
            output_format: "routing receipts".to_string(),
        },
        AgentRole {
            id: "governance-councilor".to_string(),
            title: "Governance Councilor".to_string(),
            division: "sovereign".to_string(),
            context: "Participates in the Council of 12 governance structure".to_string(),
            responsibilities: vec![
                "Vote on ASE emission proposals".to_string(),
                "Review and respond to governance actions".to_string(),
                "Represent a sector of the 24-sector governance map".to_string(),
            ],
            capabilities: vec![
                "governance".to_string(),
                "voting".to_string(),
                "ase-emission".to_string(),
            ],
            output_format: "signed governance receipts".to_string(),
        },
    ]
}
