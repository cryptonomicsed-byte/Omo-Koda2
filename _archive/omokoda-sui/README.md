# Omo-Koda Sui Move Contracts

This directory contains the Sui Move smart contracts for the Omo-Koda decentralized identity and reputation system.

## Overview

The Omo-Koda Sui contracts implement:

- **ODU Identity Management**: Decentralized identity creation and management using DNA fingerprints
- **Reputation System**: On-chain reputation tracking and tier management
- **Oracle Integration**: Secure oracle-based reputation updates

## Contracts

### ODU Contract (`odu.move`)

Core identity management contract that handles:

- ODU identity creation and registration
- Reputation score management
- Tier-based access control
- Global identity registry

## Development Setup

### Prerequisites

- Sui CLI installed
- Move package manager

### Building

```bash
cd omokoda-sui
sui move build
```

### Testing

```bash
sui move test
```

### Deployment

```bash
sui client publish --gas-budget 100000000
```

## Contract Architecture

### ODUIdentity

Represents a unique agent identity with:
- DNA fingerprint for uniqueness
- Primary Odu index (0-255)
- Reputation score and tier
- Birth timestamp
- Owner address

### ODURegistry

Global registry that:
- Maps DNA fingerprints to identity objects
- Tracks total registered identities
- Enables identity lookups

## Integration

These contracts integrate with the Rust core through:
- Identity verification via DNA fingerprints
- Reputation synchronization
- Tier-based tool access control

## Security Considerations

- Identity creation requires unique DNA fingerprints
- Reputation updates should be oracle-gated in production
- Registry operations are permissioned appropriately