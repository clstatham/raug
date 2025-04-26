//! Contains the definition of the `Edge` struct, which represents an edge in the graph.

use super::NodeIndex;

/// Represents a connection between an output and an input of two nodes.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Edge {
    /// The source node.
    pub source: NodeIndex,
    /// The target (sink) node.
    pub target: NodeIndex,

    /// The output index of the source node.
    pub source_output: u32,
    /// The input index of the target node.
    pub target_input: u32,
}
