//! A directed graph of [`Processor`]s connected by [`Edge`]s.

use std::sync::{Arc, Mutex};

use edge::Edge;
use node::{IntoInputIdx, IntoNode, IntoOutputIdx, Node, ProcessorNode};
use petgraph::{
    prelude::{Direction, EdgeRef, StableDiGraph},
    visit::DfsPostOrder,
};
use runtime::{AudioDevice, MidiPort};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    prelude::{Constant, Null, Param, Passthrough, ProcEnv},
    processor::{ProcessMode, Processor, ProcessorError},
    signal::{Signal, SignalType, buffer::SignalBuffer},
};

pub mod edge;
pub mod node;
pub mod runtime;

pub(crate) type GraphIx = u32;
/// The type of node indices.
pub type NodeIndex = petgraph::graph::NodeIndex<GraphIx>;

/// The type of the directed graph.
pub type DiGraph = StableDiGraph<ProcessorNode, Edge, GraphIx>;

/// The type of error that occurred while running a graph.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
#[error("Graph run error")]
pub enum GraphRunError {
    /// An error occurred while processing the node.
    #[error("Processor error: {0}")]
    ProcessorError(#[from] ProcessorError),

    /// An error occurred while the stream was running.
    StreamError(#[from] cpal::StreamError),

    /// An error occurred while playing the stream.
    StreamPlayError(#[from] cpal::PlayStreamError),

    /// An error occurred while pausing the stream.
    StreamPauseError(#[from] cpal::PauseStreamError),

    /// An error occurred while enumerating available audio devices.
    DevicesError(#[from] cpal::DevicesError),

    /// An error occurred while enumerating available hosts.
    Hound(#[from] hound::Error),

    /// The requested host is unavailable.
    HostUnavailable(#[from] cpal::HostUnavailable),

    /// The requested device is unavailable.
    #[error("Requested device is unavailable: {0:?}")]
    DeviceUnavailable(AudioDevice),

    /// An error occurred while retrieving the device name.
    DeviceNameError(#[from] cpal::DeviceNameError),

    /// An error occurred while retrieving the default output config.
    DefaultStreamConfigError(#[from] cpal::DefaultStreamConfigError),

    /// Output stream sample format is not supported.
    #[error("Unsupported sample format: {0}")]
    UnsupportedSampleFormat(cpal::SampleFormat),

    /// An error occurred while initializing MIDI input.
    MidirInitError(#[from] midir::InitError),

    /// The requested MIDI port is unavailable.
    #[error("Requested MIDI port is unavailable: {0:?}")]
    MidiPortUnavailable(MidiPort),

    /// An error occurred while connecting to a MIDI port.
    MidiConnectError(#[from] midir::ConnectError<midir::MidiInput>),

    /// The runtime needs to reallocate buffers.
    NeedsAlloc,

    /// An error occurred while modifying the graph.
    #[error("Graph modification error: {0}")]
    GraphModificationError(#[from] GraphConstructionError),

    /// An error occurred while sending data to the audio stream.
    #[error("Stream send error")]
    StreamSendError,

    /// An error occurred while receiving data from the audio stream.
    #[error("Stream receive error")]
    StreamReceiveError,

    /// Audio stream is not spawned.
    #[error("Audio stream not spawned")]
    StreamNotSpawned,
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

    /// Index out of bounds for the specified input.
    #[error("Input index out of bounds: {index} >= {num_inputs}")]
    InputIndexOutOfBounds {
        /// The index of the input that was out of bounds.
        index: usize,
        /// The number of inputs in the node.
        num_inputs: usize,
    },

    /// Index out of bounds for the specified output.
    #[error("Output index out of bounds: {index} >= {num_outputs}")]
    OutputIndexOutOfBounds {
        /// The index of the output that was out of bounds.
        index: usize,
        /// The number of outputs in the node.
        num_outputs: usize,
    },

    /// Filesystem error.
    #[error("Filesystem error: {0}")]
    FilesystemError(#[from] std::io::Error),
}

/// A result type for graph run operations.
pub type GraphRunResult<T> = Result<T, GraphRunError>;

/// A result type for graph construction operations.
pub type GraphConstructionResult<T> = Result<T, GraphConstructionError>;

#[derive(Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub(crate) struct GraphInner {
    pub(crate) digraph: DiGraph,

    // parameters for the graph
    params: FxHashMap<String, NodeIndex>,

    // cached input/output nodes
    input_nodes: Vec<NodeIndex>,
    output_nodes: Vec<NodeIndex>,

    // cached visitor state for graph traversal
    #[cfg_attr(feature = "serde", serde(skip))]
    visitor: DfsPostOrder<NodeIndex, FxHashSet<NodeIndex>>,
    visit_path: Vec<NodeIndex>,

    // cached strongly connected components (feedback loops)
    sccs: Vec<Vec<NodeIndex>>,

    pub(crate) sample_rate: f32,
    pub(crate) block_size: usize,
    pub(crate) max_block_size: usize,
}

impl GraphInner {
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
        let mut node = ProcessorNode::new(processor);

        // allocate for the node on-the-fly with the current sample rate and
        // block size, so that it can immediately be used
        node.allocate(self.sample_rate, self.max_block_size);
        node.resize_buffers(self.sample_rate, self.max_block_size);

        self.digraph.add_node(node)
    }

    /// Adds a parameter node to the graph.
    pub fn add_param(&mut self, param: Param) -> NodeIndex {
        let name = param.name().to_string();
        let index = self.add_processor(param);
        self.params.insert(name, index);
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

    /// Returns the indices of the audio outputs in the graph.
    #[inline]
    pub fn output_indices(&self) -> &[NodeIndex] {
        &self.output_nodes
    }

    /// Returns a reference to the runtime's output buffer for the given output index.
    #[inline]
    pub fn get_output(&self, output_index: usize) -> Option<&SignalBuffer> {
        let output_index = *self.output_indices().get(output_index)?;
        self.digraph()
            .node_weight(output_index)
            .map(|buffers| &buffers.outputs.as_ref().unwrap()[0])
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
        F: FnMut(&mut GraphInner, NodeIndex) -> Result<(), E>,
    {
        self.reset_visitor();

        for i in 0..self.visit_path.len() {
            f(self, self.visit_path[i])?;
        }

        Ok(())
    }

    /// Calls [`Processor::allocate()`] on each node in the graph.
    pub fn allocate(&mut self, sample_rate: f32, max_block_size: usize) {
        self.reset_visitor();
        self.visit(|graph, node| -> Result<(), ()> {
            graph.digraph[node].allocate(sample_rate, max_block_size);
            Ok(())
        })
        .unwrap();
        self.resize_buffers(sample_rate, max_block_size);

        self.sample_rate = sample_rate;
        self.block_size = max_block_size;
        self.max_block_size = max_block_size;
    }

    /// Calls [`Processor::resize_buffers()`] on each node in the graph.
    pub fn resize_buffers(&mut self, sample_rate: f32, block_size: usize) {
        self.visit(|graph, node| -> Result<(), ()> {
            graph.digraph[node].resize_buffers(sample_rate, block_size);
            Ok(())
        })
        .unwrap();

        self.sample_rate = sample_rate;
        self.block_size = block_size;
    }

    /// Writes a DOT representation of the graph to the provided writer, suitable for rendering with Graphviz.
    pub fn write_dot<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        write!(writer, "{:?}", petgraph::dot::Dot::new(&self.digraph))
    }

    /// Runs the audio graph for one block of samples.
    #[cfg_attr(feature = "profiling", inline(never))]
    pub fn process(&mut self) -> GraphRunResult<()> {
        for i in 0..self.sccs.len() {
            if self.sccs()[i].len() == 1 {
                let node_id = self.sccs[i][0];
                self.process_node(node_id, ProcessMode::Block)?;
            } else {
                let nodes = self.sccs[i].clone();
                for sample_index in 0..self.block_size {
                    for &node_id in &nodes {
                        self.process_node(node_id, ProcessMode::Sample(sample_index))?;
                    }
                }
            }
        }

        Ok(())
    }

    #[cfg_attr(feature = "profiling", inline(never))]
    fn process_node(&mut self, node_id: NodeIndex, mode: ProcessMode) -> GraphRunResult<()> {
        let num_inputs = self.digraph()[node_id].num_inputs();

        let mut inputs: smallvec::SmallVec<[_; 32]> = smallvec::smallvec![None; num_inputs];

        let mut buffers = self.digraph_mut()[node_id].outputs.take().unwrap();

        for (source_id, edge) in self
            .digraph()
            .edges_directed(node_id, Direction::Incoming)
            .map(|edge| (edge.source(), edge.weight()))
        {
            let source_buffers = self.digraph[source_id].outputs.as_ref().unwrap();
            let buffer = &source_buffers[edge.source_output as usize] as *const SignalBuffer;

            inputs[edge.target_input as usize] = Some(buffer);
        }

        let node = &mut self.digraph[node_id];

        for buffer in &mut buffers {
            buffer.fill_none();
        }

        let result = node.process(
            &inputs[..],
            ProcEnv {
                sample_rate: self.sample_rate,
                block_size: self.block_size,
                mode,
            },
            &mut buffers,
            mode,
        );

        if let Err(err) = result {
            let node = self.digraph().node_weight(node_id).unwrap();
            log::error!("Error processing node {}: {:?}", node.name(), err);
            return Err(err.into());
        }

        drop(inputs);

        self.digraph_mut()[node_id].outputs = Some(buffers);

        Ok(())
    }
}

/// A directed graph of [`Processor`]s connected by [`Edge`]s.
#[derive(Clone, Default)]
pub struct Graph {
    inner: Arc<Mutex<GraphInner>>,
}

impl Graph {
    /// Creates a new `GraphBuilder` with an empty graph.
    pub fn new() -> Self {
        Self::default()
    }

    pub fn allocate(&self, sample_rate: f32, block_size: usize) {
        self.with_inner(|graph| graph.allocate(sample_rate, block_size));
    }

    pub fn resize_buffers(&self, sample_rate: f32, block_size: usize) {
        self.with_inner(|graph| graph.resize_buffers(sample_rate, block_size));
    }

    pub fn sample_rate(&self) -> f32 {
        self.with_inner(|graph| graph.sample_rate)
    }

    pub fn block_size(&self) -> usize {
        self.with_inner(|graph| graph.block_size)
    }

    pub fn max_block_size(&self) -> usize {
        self.with_inner(|graph| graph.max_block_size)
    }

    pub fn process(&self) -> GraphRunResult<()> {
        self.with_inner(|graph| graph.process())
    }

    pub fn num_audio_inputs(&self) -> usize {
        self.with_inner(|graph| graph.num_audio_inputs())
    }

    pub fn num_audio_outputs(&self) -> usize {
        self.with_inner(|graph| graph.num_audio_outputs())
    }

    pub fn num_params(&self) -> usize {
        self.with_inner(|graph| graph.num_params())
    }

    pub fn param_named(&self, name: &str) -> Option<Param> {
        self.with_inner(|graph| graph.param_named(name).cloned())
    }

    pub fn param_index(&self, name: &str) -> Option<NodeIndex> {
        self.with_inner(|graph| graph.param_index(name))
    }

    /// Adds an audio input node to the graph.
    pub fn add_audio_input(&self) -> Node {
        self.with_inner(|graph| Node {
            graph: self.clone(),
            node_id: graph.add_audio_input(),
        })
    }

    /// Adds an audio output node to the graph.
    pub fn add_audio_output(&self) -> Node {
        self.with_inner(|graph| Node {
            graph: self.clone(),
            node_id: graph.add_audio_output(),
        })
    }

    /// Adds a processor node to the graph.
    pub fn add(&self, processor: impl Processor) -> Node {
        self.with_inner(|graph| Node {
            graph: self.clone(),
            node_id: graph.add_processor(processor),
        })
    }

    /// Adds a parameter node to the graph.
    pub fn add_param(&self, value: Param) -> Node {
        self.with_inner(|graph| Node {
            graph: self.clone(),
            node_id: graph.add_param(value),
        })
    }

    /// Adds a node that outputs a constant value every sample.
    pub fn constant(&self, value: impl Signal) -> Node {
        self.add(Constant::new(value))
    }

    /// Returns the number of nodes in the graph.
    pub fn node_count(&self) -> usize {
        self.with_inner(|graph| graph.digraph.node_count())
    }

    /// Returns the number of edges in the graph.
    pub fn edge_count(&self) -> usize {
        self.with_inner(|graph| graph.digraph.edge_count())
    }

    /// Runs the given closure with a reference to the graph.
    pub(crate) fn with_inner<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut GraphInner) -> R,
    {
        f(&mut self.inner.lock().unwrap())
    }

    /// Connects the given output of one node to the given input of another node.
    #[track_caller]
    #[inline]
    pub fn connect(
        &self,
        from: impl IntoNode,
        from_output: impl IntoOutputIdx,
        to: impl IntoNode,
        to_input: impl IntoInputIdx,
    ) {
        let from = from.into_node(self);
        let to = to.into_node(self);
        let from_output = from_output.into_output_idx(&from);
        let to_input = to_input.into_input_idx(&to);
        self.with_inner(|graph| graph.connect(from.id(), from_output, to.id(), to_input))
            .unwrap();
    }

    /// Disconnects the given output of one node from the given input of another node.
    #[track_caller]
    #[inline]
    pub fn disconnect(
        &self,
        from: impl IntoNode,
        from_output: impl IntoOutputIdx,
        to: impl IntoNode,
        to_input: impl IntoInputIdx,
    ) {
        let from = from.into_node(self);
        let to = to.into_node(self);
        let from_output = from_output.into_output_idx(&from);
        let to_input = to_input.into_input_idx(&to);
        self.with_inner(|graph| graph.disconnect(from.id(), from_output, to.id(), to_input));
    }

    /// Disconnects all inputs to the given node.
    #[track_caller]
    #[inline]
    pub fn disconnect_all_inputs(&self, node: impl IntoNode) {
        let node = node.into_node(self);
        self.with_inner(|graph| graph.disconnect_all_inputs(node.id()));
    }

    /// Disconnects all outputs from the given node.
    #[track_caller]
    #[inline]
    pub fn disconnect_all_outputs(&self, node: impl IntoNode) {
        let node = node.into_node(self);
        self.with_inner(|graph| graph.disconnect_all_outputs(node.id()));
    }

    /// Disconnects all inputs and outputs from the given node.
    #[track_caller]
    #[inline]
    pub fn disconnect_all(&self, node: impl IntoNode) {
        let node = node.into_node(self);
        self.with_inner(|graph| graph.disconnect_all(node.id()));
    }

    /// Writes a DOT representation of the graph to the provided writer, suitable for rendering with Graphviz.
    pub fn write_dot<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        self.with_inner(|graph| graph.write_dot(writer))
    }
}
