//! Causal Memory DAG — memory linked by causality, not semantic similarity.
//! No vector DB. Each node links to its causal parents.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A node in the causal memory graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemNode {
    pub id: String,
    pub content: String,
    pub timestamp: u64,
    /// IDs of causal parent nodes (causes of this memory).
    pub parent_ids: Vec<String>,
}

impl MemNode {
    pub fn new(id: String, content: String, timestamp: u64) -> Self {
        Self {
            id,
            content,
            timestamp,
            parent_ids: Vec::new(),
        }
    }

    pub fn with_parents(mut self, parents: Vec<String>) -> Self {
        self.parent_ids = parents;
        self
    }
}

/// Directed Acyclic Graph of causal memory.
/// Traversal is causal (cause → effect), not semantic.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CausalMemoryDag {
    nodes: HashMap<String, MemNode>,
}

impl CausalMemoryDag {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, node: MemNode) {
        self.nodes.insert(node.id.clone(), node);
    }

    pub fn get(&self, id: &str) -> Option<&MemNode> {
        self.nodes.get(id)
    }

    pub fn causal_chain(&self, id: &str) -> Vec<&MemNode> {
        let mut chain = Vec::new();
        let mut current = id;
        let mut visited = std::collections::HashSet::new();
        loop {
            if visited.contains(current) {
                break;
            }
            visited.insert(current);
            if let Some(node) = self.nodes.get(current) {
                chain.push(node);
                if let Some(parent) = node.parent_ids.first() {
                    current = parent.as_str();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        chain
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_retrieve_node() {
        let mut dag = CausalMemoryDag::new();
        let node = MemNode::new("n1".to_string(), "first thought".to_string(), 0);
        dag.insert(node);
        assert!(dag.get("n1").is_some());
        assert_eq!(dag.len(), 1);
    }

    #[test]
    fn causal_chain_follows_parents() {
        let mut dag = CausalMemoryDag::new();
        dag.insert(MemNode::new("root".to_string(), "origin".to_string(), 0));
        dag.insert(
            MemNode::new("child".to_string(), "consequence".to_string(), 1)
                .with_parents(vec!["root".to_string()]),
        );
        dag.insert(
            MemNode::new("grandchild".to_string(), "effect".to_string(), 2)
                .with_parents(vec!["child".to_string()]),
        );

        let chain = dag.causal_chain("grandchild");
        assert_eq!(chain.len(), 3);
        assert_eq!(chain[0].id, "grandchild");
        assert_eq!(chain[2].id, "root");
    }

    #[test]
    fn dag_handles_missing_nodes() {
        let dag = CausalMemoryDag::new();
        assert!(dag.get("nonexistent").is_none());
        assert!(dag.causal_chain("nonexistent").is_empty());
    }
}
