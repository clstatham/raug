//! Contains the definition of the `Edge` struct, which represents an edge in the graph.

/// Represents a connection between an output and an input of two nodes.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Edge {
    /// The output index of the source node.
    pub source_output: u32,
    /// The input index of the target node.
    pub target_input: u32,

    /// The name of the output of the source node.
    pub source_output_name: Option<String>,
    /// The name of the input of the target node.
    pub target_input_name: Option<String>,
}

impl std::fmt::Debug for Edge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let source_output = if let Some(name) = &self.source_output_name {
            name
        } else {
            &self.source_output.to_string()
        };
        let target_input = if let Some(name) = &self.target_input_name {
            name
        } else {
            &self.target_input.to_string()
        };
        write!(f, "{}->{}", source_output, target_input)
    }
}
