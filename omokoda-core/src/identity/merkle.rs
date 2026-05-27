//! Merkle root verification for agent identity integrity.
//! From bipon39: every agent identity commit includes Merkle root.

use blake3;

/// A Merkle tree over identity commitment fields.
/// Used to verify agent identity integrity at key milestones.
pub struct IdentityMerkleTree {
    leaves: Vec<[u8; 32]>,
}

impl IdentityMerkleTree {
    pub fn new() -> Self {
        Self { leaves: Vec::new() }
    }

    /// Add a field to the Merkle tree (hashes the value).
    pub fn add_field(&mut self, field: &[u8]) {
        let hash = blake3::hash(field);
        let mut leaf = [0u8; 32];
        leaf.copy_from_slice(hash.as_bytes());
        self.leaves.push(leaf);
    }

    /// Compute the Merkle root over all added fields.
    pub fn root(&self) -> [u8; 32] {
        if self.leaves.is_empty() {
            return [0u8; 32];
        }
        let mut layer = self.leaves.clone();
        while layer.len() > 1 {
            let mut next = Vec::new();
            let mut i = 0;
            while i < layer.len() {
                let left = &layer[i];
                let right = if i + 1 < layer.len() { &layer[i + 1] } else { left };
                let mut hasher = blake3::Hasher::new();
                hasher.update(left);
                hasher.update(right);
                let hash = hasher.finalize();
                let mut node = [0u8; 32];
                node.copy_from_slice(hash.as_bytes());
                next.push(node);
                i += 2;
            }
            layer = next;
        }
        layer[0]
    }

    pub fn root_hex(&self) -> String {
        hex::encode(self.root())
    }

    /// Build a standard identity Merkle tree from soul fields.
    pub fn from_soul(
        agent_id: &str,
        birth_timestamp: u64,
        odu_index: u8,
        dna_fingerprint: &str,
    ) -> Self {
        let mut tree = Self::new();
        tree.add_field(agent_id.as_bytes());
        tree.add_field(&birth_timestamp.to_le_bytes());
        tree.add_field(&[odu_index]);
        tree.add_field(dna_fingerprint.as_bytes());
        tree
    }
}

impl Default for IdentityMerkleTree {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_tree_returns_zero_root() {
        let tree = IdentityMerkleTree::new();
        assert_eq!(tree.root(), [0u8; 32]);
    }

    #[test]
    fn single_field_root_is_deterministic() {
        let mut t1 = IdentityMerkleTree::new();
        t1.add_field(b"hello");
        let mut t2 = IdentityMerkleTree::new();
        t2.add_field(b"hello");
        assert_eq!(t1.root(), t2.root());
    }

    #[test]
    fn different_fields_produce_different_roots() {
        let mut t1 = IdentityMerkleTree::new();
        t1.add_field(b"agent_a");
        let mut t2 = IdentityMerkleTree::new();
        t2.add_field(b"agent_b");
        assert_ne!(t1.root(), t2.root());
    }

    #[test]
    fn soul_tree_is_deterministic() {
        let r1 = IdentityMerkleTree::from_soul("oracle", 1_700_000_000, 42, "dna_abc").root();
        let r2 = IdentityMerkleTree::from_soul("oracle", 1_700_000_000, 42, "dna_abc").root();
        assert_eq!(r1, r2);
    }

    #[test]
    fn root_hex_is_64_chars() {
        let tree = IdentityMerkleTree::from_soul("a", 1, 0, "d");
        assert_eq!(tree.root_hex().len(), 64);
    }
}
