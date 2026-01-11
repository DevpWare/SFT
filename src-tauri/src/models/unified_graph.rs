use serde::{Deserialize, Serialize};
use super::{UnifiedNode, UnifiedEdge};

/// Graph metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GraphMetadata {
    /// Project name
    pub project_name: String,

    /// Project root path
    pub root_path: String,

    /// Primary language
    pub language: String,

    /// Total files scanned
    pub total_files: usize,

    /// Total lines of code
    pub total_lines: Option<u64>,

    /// Scan timestamp
    pub scanned_at: Option<String>,

    /// Parser version used
    pub parser_version: String,
}

/// Complete unified graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedGraph {
    /// All nodes
    pub nodes: Vec<UnifiedNode>,

    /// All edges
    pub edges: Vec<UnifiedEdge>,

    /// Graph metadata
    pub metadata: GraphMetadata,
}

impl UnifiedGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            metadata: GraphMetadata::default(),
        }
    }

    pub fn with_metadata(mut self, metadata: GraphMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn add_node(&mut self, node: UnifiedNode) {
        self.nodes.push(node);
    }

    pub fn add_edge(&mut self, edge: UnifiedEdge) {
        self.edges.push(edge);
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Find node by ID
    pub fn find_node(&self, id: &str) -> Option<&UnifiedNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// Get all edges from a node
    pub fn edges_from(&self, node_id: &str) -> Vec<&UnifiedEdge> {
        self.edges.iter().filter(|e| e.source == node_id).collect()
    }

    /// Get all edges to a node
    pub fn edges_to(&self, node_id: &str) -> Vec<&UnifiedEdge> {
        self.edges.iter().filter(|e| e.target == node_id).collect()
    }

    /// Calculate in-degree for a node
    pub fn in_degree(&self, node_id: &str) -> usize {
        self.edges_to(node_id).len()
    }

    /// Calculate out-degree for a node
    pub fn out_degree(&self, node_id: &str) -> usize {
        self.edges_from(node_id).len()
    }
}

impl Default for UnifiedGraph {
    fn default() -> Self {
        Self::new()
    }
}
