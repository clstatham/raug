//! A directed graph of [`Processor`]s connected by [`Edge`]s.

use asset::{Asset, Assets};
use edge::Edge;
use node::ProcessorNode;
use petgraph::{
    prelude::{Direction, EdgeRef, StableDiGraph},
    visit::DfsPostOrder,
};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    midi::MidiMessage,
    prelude::{Null, Param, Passthrough},
    processor::{Processor, ProcessorError},
    signal::{Float, SignalType},
};

pub mod asset;
pub mod edge;
pub mod node;

/// The type of graph indices.
pub type GraphIx = u32;
/// The type of node indices.
pub type NodeIndex = petgraph::graph::NodeIndex<GraphIx>;

/// The type of the directed graph.
pub type DiGraph = StableDiGraph<ProcessorNode, Edge, GraphIx>;

/// An error that occurred while running a graph.
#[derive(Debug, Clone, thiserror::Error)]
#[error("Graph run error at node {} ({}): {signal_type:?}", node_index.index(), node_processor)]
pub struct GraphRunError {
    /// The index of the node where the error occurred.
    pub node_index: NodeIndex,
    /// The name of the processor of the node where the error occurred.
    pub node_processor: String,
    /// The type of error that occurred.
    pub signal_type: GraphRunErrorType,
}

/// The type of error that occurred while running a graph.
#[derive(Debug, Clone, thiserror::Error)]
#[non_exhaustive]
pub enum GraphRunErrorType {
    /// An error occurred while processing the node.
    #[error("Processor error: {0}")]
    ProcessorError(#[from] ProcessorError),
}

/// An error that occurred while constructing a graph.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum GraphConstructionError {
    /// Attempted to connect nodes from different graphs.
    #[error("Cannot connect nodes from different graphs")]
    MismatchedGraphs,

    /// Attempted to perform an invalid operation on a node with multiple outputs.
    #[error("Operation `{op}` invalid: Node type `{signal_type}` has multiple outputs")]
    NodeHasMultipleOutputs {
        /// The operation that was attempted.
        op: String,
        /// The type of the node.
        signal_type: String,
    },

    /// Filesystem error.
    #[error("Filesystem error: {0}")]
    FilesystemError(#[from] std::io::Error),
}

/// A result type for graph run operations.
pub type GraphRunResult<T> = Result<T, GraphRunError>;

/// A result type for graph construction operations.
pub type GraphConstructionResult<T> = Result<T, GraphConstructionError>;

/// A directed graph of [`Processor`]s connected by [`Edge`]s.
#[derive(Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Graph {
    pub(crate) digraph: DiGraph,

    // assets in the graph
    pub(crate) assets: Assets,

    // parameters for the graph
    params: FxHashMap<String, NodeIndex>,

    // MIDI input params
    midi_params: Vec<NodeIndex>,

    // cached input/output nodes
    input_nodes: Vec<NodeIndex>,
    output_nodes: Vec<NodeIndex>,

    // cached visitor state for graph traversal
    #[cfg_attr(feature = "serde", serde(skip))]
    visitor: DfsPostOrder<NodeIndex, FxHashSet<NodeIndex>>,
    visit_path: Vec<NodeIndex>,

    // cached strongly connected components (feedback loops)
    sccs: Vec<Vec<NodeIndex>>,
}

impl Graph {
    /// Creates a new, empty graph.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a reference to the underlying [`DiGraph`].
    #[inline]
    pub fn digraph(&self) -> &DiGraph {
        &self.digraph
    }

    /// Returns a mutable reference to the underlying [`DiGraph`].
    #[inline]
    pub fn digraph_mut(&mut self) -> &mut DiGraph {
        &mut self.digraph
    }

    /// Returns a reference to the assets in the graph.
    #[inline]
    pub fn assets(&self) -> &Assets {
        &self.assets
    }

    /// Adds an asset to the graph.
    pub fn add_asset(&mut self, name: impl Into<String>, asset: Asset) {
        self.assets.insert(name.into(), asset);
    }

    /// Adds an audio input node to the graph.
    pub fn add_audio_input(&mut self) -> NodeIndex {
        let idx = self.digraph.add_node(ProcessorNode::new(Null));
        self.input_nodes.push(idx);
        idx
    }

    /// Adds an audio output node to the graph.
    pub fn add_audio_output(&mut self) -> NodeIndex {
        let idx = self
            .digraph
            .add_node(ProcessorNode::new(Passthrough::new(SignalType::Float)));
        self.output_nodes.push(idx);
        idx
    }

    /// Adds a processor node to the graph.
    pub fn add_processor(&mut self, processor: impl Processor) -> NodeIndex {
        self.digraph.add_node(ProcessorNode::new(processor))
    }

    /// Adds a parameter node to the graph.
    pub fn add_param(&mut self, param: Param) -> NodeIndex {
        let name = param.name().to_string();
        let index = self.add_processor(param);
        self.params.insert(name, index);
        index
    }

    /// Adds a MIDI input node to the graph.
    pub fn add_midi_input(&mut self, name: impl Into<String>) -> NodeIndex {
        let param = Param::new::<MidiMessage>(name, None);
        let index = self.add_param(param);
        self.midi_params.push(index);
        index
    }

    /// Connects two nodes in the graph.
    ///
    /// If the edge already exists, this function does nothing.
    ///
    /// If the target node already has an incoming edge at the target input, the existing edge is removed.
    pub fn connect(
        &mut self,
        source: NodeIndex,
        source_output: u32,
        target: NodeIndex,
        target_input: u32,
    ) -> Result<(), GraphConstructionError> {
        // check if there's already a connection to the target input
        if let Some(edge) = self
            .digraph
            .edges_directed(target, Direction::Incoming)
            .find(|edge| edge.weight().target_input == target_input)
        {
            // remove the existing edge
            self.digraph.remove_edge(edge.id()).unwrap();
        }

        let source_output_name = self.digraph[source].output_spec()[source_output as usize]
            .name
            .clone();

        let target_input_name = self.digraph[target].input_spec()[target_input as usize]
            .name
            .clone();

        self.digraph.add_edge(
            source,
            target,
            Edge {
                source_output,
                target_input,
                source_output_name: Some(source_output_name),
                target_input_name: Some(target_input_name),
            },
        );

        self.reset_visitor();

        self.detect_sccs();

        Ok(())
    }

    /// Disconnects two nodes in the graph at the specified input and output indices.
    ///
    /// Does nothing if the edge does not exist.
    pub fn disconnect(
        &mut self,
        source: NodeIndex,
        source_output: u32,
        target: NodeIndex,
        target_input: u32,
    ) {
        let edge = self
            .digraph
            .edges_directed(target, Direction::Incoming)
            .find(|edge| {
                let weight = edge.weight();
                edge.source() == source
                    && weight.source_output == source_output
                    && weight.target_input == target_input
            });

        if let Some(edge) = edge {
            self.digraph.remove_edge(edge.id()).unwrap();
            self.reset_visitor();
            self.detect_sccs();
        }
    }

    /// Disconnects all inputs to the specified node.
    pub fn disconnect_all_inputs(&mut self, node: NodeIndex) {
        let incoming_edges = self
            .digraph
            .edges_directed(node, Direction::Incoming)
            .map(|edge| edge.id())
            .collect::<Vec<_>>();
        for edge in incoming_edges {
            self.digraph.remove_edge(edge).unwrap();
            self.reset_visitor();
            self.detect_sccs();
        }
    }

    /// Disconnects all outputs from the specified node.
    pub fn disconnect_all_outputs(&mut self, node: NodeIndex) {
        let outgoing_edges = self
            .digraph
            .edges_directed(node, Direction::Outgoing)
            .map(|edge| edge.id())
            .collect::<Vec<_>>();
        for edge in outgoing_edges {
            self.digraph.remove_edge(edge).unwrap();
            self.reset_visitor();
            self.detect_sccs();
        }
    }

    /// Disconnects all inputs and outputs from the specified node.
    pub fn disconnect_all(&mut self, node: NodeIndex) {
        self.disconnect_all_inputs(node);
        self.disconnect_all_outputs(node);
    }

    /// Returns the number of audio inputs in the graph.
    #[inline]
    pub fn num_audio_inputs(&self) -> usize {
        self.input_nodes.len()
    }

    /// Returns the number of audio outputs in the graph.
    #[inline]
    pub fn num_audio_outputs(&self) -> usize {
        self.output_nodes.len()
    }

    /// Returns the name of the given node's processor.
    #[inline]
    pub fn node_name(&self, node: NodeIndex) -> &str {
        self.digraph[node].name()
    }

    /// Returns the number of parameters in the graph.
    #[inline]
    pub fn num_params(&self) -> usize {
        self.params.len()
    }

    /// Returns the index of the parameter with the specified name.
    #[inline]
    pub fn param_index(&self, name: &str) -> Option<NodeIndex> {
        self.params.get(name).copied()
    }

    /// Returns the parameter with the specified name.
    #[inline]
    pub fn param_named(&self, name: &str) -> Option<&Param> {
        self.param_index(name)
            .map(|idx| (*self.digraph[idx].processor()).downcast_ref().unwrap())
    }

    /// Returns the index of the MIDI input with the specified name.
    #[inline]
    pub fn midi_input_index(&self, name: &str) -> Option<NodeIndex> {
        self.params
            .get(name)
            .copied()
            .filter(|&idx| self.midi_params.contains(&idx))
    }

    /// Returns an iterator over the MIDI input parameters in the graph.
    #[inline]
    pub fn midi_input_iter(&self) -> impl Iterator<Item = (&str, Param)> + '_ {
        self.params
            .iter()
            .filter(|(name, _)| self.midi_params.contains(self.params.get(*name).unwrap()))
            .map(|(name, idx)| {
                (
                    name.as_str(),
                    (*self.digraph[*idx].processor())
                        .downcast_ref::<Param>()
                        .unwrap()
                        .clone(),
                )
            })
    }

    /// Returns the indices of the audio inputs in the graph.
    #[inline]
    pub fn input_indices(&self) -> &[NodeIndex] {
        &self.input_nodes
    }

    /// Returns the indices of the audio outputs in the graph.
    #[inline]
    pub fn output_indices(&self) -> &[NodeIndex] {
        &self.output_nodes
    }

    #[inline]
    pub(crate) fn sccs(&self) -> &[Vec<NodeIndex>] {
        &self.sccs
    }

    #[inline]
    pub(crate) fn detect_sccs(&mut self) {
        self.sccs = petgraph::algo::kosaraju_scc(&self.digraph);
        self.sccs.reverse();
    }

    #[inline]
    pub(crate) fn reset_visitor(&mut self) {
        if self.visit_path.capacity() < self.digraph.node_count() {
            self.visit_path = Vec::with_capacity(self.digraph.node_count());
        }
        self.visit_path.clear();
        self.visitor.discovered.clear();
        self.visitor.stack.clear();
        self.visitor.finished.clear();

        for node in self.digraph.externals(Direction::Incoming) {
            self.visitor.stack.push(node);
        }
        while let Some(node) = self.visitor.next(&self.digraph) {
            self.visit_path.push(node);
        }
        self.visit_path.reverse();
    }

    /// Calls the provided closure on each node in the graph in topological order.
    pub fn visit<F, E>(&mut self, mut f: F) -> Result<(), E>
    where
        F: FnMut(&mut Graph, NodeIndex) -> Result<(), E>,
    {
        self.reset_visitor();

        for i in 0..self.visit_path.len() {
            f(self, self.visit_path[i])?;
        }

        Ok(())
    }

    /// Calls [`Processor::allocate()`] on each node in the graph.
    pub fn allocate(&mut self, sample_rate: Float, max_block_size: usize) {
        self.visit(|graph, node| -> Result<(), ()> {
            graph.digraph[node].allocate(sample_rate, max_block_size);
            Ok(())
        })
        .unwrap();
    }

    /// Calls [`Processor::resize_buffers()`] on each node in the graph.
    pub fn resize_buffers(&mut self, sample_rate: Float, block_size: usize) {
        self.visit(|graph, node| -> Result<(), ()> {
            graph.digraph[node].resize_buffers(sample_rate, block_size);
            Ok(())
        })
        .unwrap();
    }

    /// Writes a DOT representation of the graph to the provided writer, suitable for rendering with Graphviz.
    pub fn write_dot<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        write!(writer, "{:?}", petgraph::dot::Dot::new(&self.digraph))
    }
}
