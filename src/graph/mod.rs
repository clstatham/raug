//! A directed graph of [`Processor`]s connected by [`Edge`]s.

use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    thread::JoinHandle,
    time::Duration,
};

use atomic_time::AtomicDuration;
use crossbeam_channel::Sender;
use edge::Edge;
use node::{
    IntoInputIdx, IntoNode, IntoOutputIdx, IntoOutputs, Node, ProcessNodeError, ProcessorNode,
};
use petgraph::{
    prelude::{Direction, EdgeRef, StableDiGraph},
    visit::DfsPostOrder,
};
use runtime::{AudioDevice, AudioOut};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    prelude::{Constant, Null, Passthrough, ProcEnv},
    processor::{Processor, io::ProcessMode},
    signal::{Signal, type_erased::AnyBuffer},
};

pub mod edge;
pub mod node;
pub mod runtime;
pub mod sub_graph;

/// The inner type of node indices.
pub(crate) type GraphIx = u32;
/// The type of node indices.
pub type NodeIndex = petgraph::graph::NodeIndex<GraphIx>;

/// The type of the directed graph.
pub type DiGraph = StableDiGraph<ProcessorNode, Edge, GraphIx>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum VisitorReset {
    #[default]
    Ready,
    NeedsReset,
}

/// The type of error that occurred while running a graph.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
#[error("Graph run error")]
pub enum GraphRunError {
    #[error("Unknown audio backend: {0}")]
    UnknownBackend(String),

    /// An error occurred while processing the node.
    ProcessorNodeError(#[from] ProcessNodeError),

    /// An error occurred while the stream was running.
    StreamError(#[from] cpal::StreamError),

    /// An error occurred while playing the stream.
    StreamPlayError(#[from] cpal::PlayStreamError),

    /// An error occurred while pausing the stream.
    StreamPauseError(#[from] cpal::PauseStreamError),

    /// An error occurred while enumerating available audio devices.
    DevicesError(#[from] cpal::DevicesError),

    /// An error occurred reading or writing the WAV file.
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

#[derive(Default)]
pub struct GraphInner {
    pub(crate) digraph: DiGraph,

    // cached input/output nodes
    input_nodes: Vec<NodeIndex>,
    output_nodes: Vec<NodeIndex>,

    // cached visitor state for graph traversal
    visitor: DfsPostOrder<NodeIndex, FxHashSet<NodeIndex>>,
    visit_path: Vec<NodeIndex>,
    visitor_reset: VisitorReset,

    // cached strongly connected components (feedback loops)
    sccs: Vec<Vec<NodeIndex>>,

    pub(crate) sample_rate: f32,
    pub(crate) block_size: usize,
    pub(crate) max_block_size: usize,
}

impl GraphInner {
    pub fn new(inputs: usize, outputs: usize) -> Self {
        let mut this = Self::default();
        for _ in 0..inputs {
            this.add_audio_input();
        }
        for _ in 0..outputs {
            this.add_audio_output();
        }
        this
    }

    /// Adds an audio input node to the graph.
    pub fn add_audio_input(&mut self) -> NodeIndex {
        let idx = self.digraph.add_node(ProcessorNode::new(Null::default()));
        self.input_nodes.push(idx);
        idx
    }

    /// Adds an audio output node to the graph.
    pub(crate) fn add_audio_output(&mut self) -> NodeIndex {
        let idx = self
            .digraph
            .add_node(ProcessorNode::new(Passthrough::<f32>::default()));
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

        let node = self.digraph.add_node(node);
        self.visitor_reset = VisitorReset::NeedsReset;

        node
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
    ) {
        // check if there's already a connection to the target input
        if let Some(edge) = self
            .digraph
            .edges_directed(target, Direction::Incoming)
            .find(|edge| edge.weight().target_input == target_input)
        {
            // remove the existing edge
            self.digraph.remove_edge(edge.id()).unwrap();
        }

        self.digraph.add_edge(
            source,
            target,
            Edge {
                source,
                target,
                source_output,
                target_input,
            },
        );

        self.visitor_reset = VisitorReset::NeedsReset;
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
        }
        self.visitor_reset = VisitorReset::NeedsReset;
    }

    /// Disconnects all inputs to the specified node.
    pub fn disconnect_all_inputs(&mut self, node: NodeIndex) -> Vec<Edge> {
        let incoming_edges = self
            .digraph
            .edges_directed(node, Direction::Incoming)
            .map(|edge| edge.id())
            .collect::<Vec<_>>();
        let mut edges_removed = Vec::new();
        for edge in incoming_edges {
            let edge = self.digraph.remove_edge(edge).unwrap();
            edges_removed.push(edge);
        }
        self.visitor_reset = VisitorReset::NeedsReset;
        edges_removed
    }

    /// Disconnects all outputs from the specified node.
    pub fn disconnect_all_outputs(&mut self, node: NodeIndex) -> Vec<Edge> {
        let outgoing_edges = self
            .digraph
            .edges_directed(node, Direction::Outgoing)
            .map(|edge| edge.id())
            .collect::<Vec<_>>();
        let mut edges_removed = Vec::new();
        for edge in outgoing_edges {
            let edge = self.digraph.remove_edge(edge).unwrap();
            edges_removed.push(edge);
        }
        self.visitor_reset = VisitorReset::NeedsReset;
        edges_removed
    }

    /// Disconnects all inputs and outputs from the specified node.
    pub fn disconnect_all(&mut self, node: NodeIndex) -> Vec<Edge> {
        let mut edges_removed = Vec::new();
        edges_removed.extend(self.disconnect_all_inputs(node));
        edges_removed.extend(self.disconnect_all_outputs(node));
        self.visitor_reset = VisitorReset::NeedsReset;
        edges_removed
    }

    pub(crate) fn gc_nodes(&mut self) {
        let nodes: FxHashSet<NodeIndex> = self.digraph.node_indices().collect();
        let mut removed = FxHashSet::default();
        let mut again = false;
        while !again {
            again = false;
            for node in nodes.iter() {
                if removed.contains(node) {
                    continue;
                }
                if !self.output_nodes.contains(node)
                    && self
                        .digraph
                        .edges_directed(*node, Direction::Outgoing)
                        .count()
                        == 0
                {
                    self.digraph.remove_node(*node);
                    removed.insert(node);
                    self.visitor_reset = VisitorReset::NeedsReset;
                    again = true;
                }
            }
        }
    }

    pub fn replace_node_gracefully(
        &mut self,
        replaced: NodeIndex,
        replacement: NodeIndex,
    ) -> NodeIndex {
        let replaced_outputs = self.disconnect_all_outputs(replaced);

        for edge in replaced_outputs {
            let Edge {
                target,
                source_output,
                target_input,
                ..
            } = edge;

            if source_output < self.digraph[replacement].num_outputs() as u32 {
                self.connect(replacement, source_output, target, target_input);
            } else {
                // leave it disconnected
            }
        }

        self.visitor_reset = VisitorReset::NeedsReset;
        self.gc_nodes();

        replacement
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

    /// Returns a mutable reference to the graph's input buffer for the given input index.
    #[inline]
    pub fn get_input_mut(&mut self, input_index: usize) -> Option<&mut [f32]> {
        let input_index = *self.input_indices().get(input_index)?;
        self.digraph
            .node_weight_mut(input_index)
            .map(|node| node.outputs[0].as_mut_slice::<f32>().unwrap())
    }

    /// Returns a reference to the graph's output buffer for the given output index.
    #[inline]
    pub fn get_output(&self, output_index: usize) -> Option<&[f32]> {
        let output_index = *self.output_indices().get(output_index)?;
        self.digraph
            .node_weight(output_index)
            .map(|buffers| buffers.outputs[0].as_slice::<f32>().unwrap())
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
        if self.visitor_reset == VisitorReset::Ready {
            return;
        }
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
        self.detect_sccs();
        self.visitor_reset = VisitorReset::Ready;
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
        let mut node_names = FxHashMap::default();
        let mut graph = dot_graph::Graph::new("Raug", dot_graph::Kind::Digraph);
        for (i, node) in self.digraph.node_indices().enumerate() {
            let node_name = format!("_{}_{}", i, self.digraph[node].name());
            graph.add_node(dot_graph::Node::new(&node_name).label(self.digraph[node].name()));
            node_names.insert(node, node_name);
        }

        for edge in self.digraph.edge_weights() {
            let source = &self.digraph[edge.source];
            let target = &self.digraph[edge.target];
            let source_name = &node_names[&edge.source];
            let target_name = &node_names[&edge.target];
            let output_name = &source.output_spec()[edge.source_output as usize].name;
            let input_name = &target.input_spec()[edge.target_input as usize].name;
            let edge_name = format!("{}->{}", output_name, input_name);
            let edge = dot_graph::Edge::new(source_name, target_name, &edge_name)
                .end_arrow(dot_graph::Arrow::normal());
            graph.add_edge(edge);
        }

        write!(writer, "{}", graph.to_dot_string()?)
    }

    /// Runs the audio graph for one block of samples.
    #[cfg_attr(feature = "profiling", inline(never))]
    pub fn process(&mut self) -> GraphRunResult<()> {
        self.reset_visitor();

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
        let mut inputs: [_; 32] = [None; 32];

        for (source_id, edge) in self
            .digraph
            .edges_directed(node_id, Direction::Incoming)
            .map(|edge| (edge.source(), edge.weight()))
        {
            let source_buffers = &self.digraph[source_id].outputs;
            let buffer = &source_buffers[edge.source_output as usize] as *const AnyBuffer;

            inputs[edge.target_input as usize] = Some(buffer);
        }

        let node = &mut self.digraph[node_id];

        node.process(
            &inputs[..],
            ProcEnv {
                sample_rate: self.sample_rate,
                block_size: self.block_size,
                mode,
            },
        )?;

        Ok(())
    }
}

/// A directed graph of [`Processor`]s connected by [`Edge`]s.
#[derive(Clone, Default)]
pub struct Graph {
    inner: Arc<Mutex<GraphInner>>,
}

impl Graph {
    /// Creates a new empty `Graph`.
    pub fn new(inputs: usize, outputs: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(GraphInner::new(inputs, outputs))),
        }
    }

    /// Returns `true` if the [`Graph`] is the same graph as `other`.
    /// [`Graph`]s are internally reference counted, so this is implemented using [`Arc::ptr_eq`].
    pub fn is_same_graph(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }

    /// Allocates internal buffers for the graph with the given sample rate and block size.
    pub fn allocate(&self, sample_rate: f32, block_size: usize) {
        self.with_inner(|graph| graph.allocate(sample_rate, block_size));
    }

    /// Resizes the buffers of the graph for the given sample rate and block size.
    pub fn resize_buffers(&self, sample_rate: f32, block_size: usize) {
        self.with_inner(|graph| graph.resize_buffers(sample_rate, block_size));
    }

    /// Returns the sample rate of the graph.
    pub fn sample_rate(&self) -> f32 {
        self.with_inner(|graph| graph.sample_rate)
    }

    /// Returns the block size of the graph.
    pub fn block_size(&self) -> usize {
        self.with_inner(|graph| graph.block_size)
    }

    /// Returns the maximum block size of the graph.
    pub fn max_block_size(&self) -> usize {
        self.with_inner(|graph| graph.max_block_size)
    }

    /// Processes the graph for one block of samples.
    pub fn process(&self) -> GraphRunResult<()> {
        self.with_inner(|graph| graph.process())
    }

    /// Returns the number of audio inputs in the graph.
    pub fn num_audio_inputs(&self) -> usize {
        self.with_inner(|graph| graph.num_audio_inputs())
    }

    /// Returns the number of audio outputs in the graph.
    pub fn num_audio_outputs(&self) -> usize {
        self.with_inner(|graph| graph.num_audio_outputs())
    }

    /// Runs the specified closure on the input buffer at the given index. This can be used to fill the buffer manually, for instance.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of range for the number of inputs in the graph.
    pub fn map_input<F, R>(&mut self, input_index: usize, f: F) -> R
    where
        F: FnOnce(&mut [f32]) -> R,
    {
        self.with_inner(|graph| f(graph.get_input_mut(input_index).unwrap()))
    }

    /// Runs the specified closure on the output buffer at the given index. This can be used to copy the buffer to something else, for instance.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of range for the number of outputs in the graph.
    pub fn map_output<F, R>(&mut self, output_index: usize, f: F) -> R
    where
        F: FnOnce(&[f32]) -> R,
    {
        self.with_inner(|graph| f(graph.get_output(output_index).unwrap()))
    }

    /// Adds an audio input node to the graph.
    pub fn adc(&self) -> Node {
        let id = self.with_inner(|graph| graph.add_audio_input());
        Node::new(self.clone(), id)
    }

    /// Adds an audio output node to the graph.
    pub fn dac(&self, inputs: impl IntoOutputs) {
        let inputs = inputs.into_outputs(self);
        self.with_inner(|graph| {
            for (o, i) in graph.output_nodes.clone().into_iter().zip(inputs) {
                graph.connect(i.node_id, i.output_index, o, 0);
            }
        });
    }

    /// Adds a processor node to the graph.
    pub fn node(&self, processor: impl Processor) -> Node {
        let id = self.with_inner(|graph| graph.add_processor(processor));
        Node::new(self.clone(), id)
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
    pub fn with_inner<F, R>(&self, f: F) -> R
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
        self.with_inner(|graph| graph.connect(from.id(), from_output, to.id(), to_input));
    }

    #[inline]
    #[track_caller]
    pub(crate) fn connect_raw(
        &self,
        from_id: NodeIndex,
        from_output: u32,
        to_id: NodeIndex,
        to_input: u32,
    ) {
        self.with_inner(|graph| graph.connect(from_id, from_output, to_id, to_input));
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

    #[track_caller]
    #[inline]
    pub fn replace_node_gracefully(
        &self,
        replaced: impl IntoNode,
        replacement: impl IntoNode,
    ) -> Node {
        let target = replaced.into_node(self);
        let replacement = replacement.into_node(self);
        let idx =
            self.with_inner(|graph| graph.replace_node_gracefully(target.id(), replacement.id()));
        Node::new(self.clone(), idx)
    }

    /// Writes a DOT representation of the graph to the provided writer, suitable for rendering with Graphviz.
    pub fn write_dot<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        self.with_inner(|graph| graph.write_dot(writer))
    }

    /// Creates a new [`Node`] that outputs a constant value.
    pub fn constant<T: Signal + Default + Clone>(&self, value: T) -> Node {
        self.node(Constant::new(value))
    }

    pub fn play(&self, mut output_stream: impl AudioOut) -> GraphRunResult<RunningGraph> {
        let graph = self.clone();
        let samples_written = Arc::new(AtomicUsize::new(0));
        let samples_written_clone = samples_written.clone();
        let duration_written = Arc::new(AtomicDuration::new(Duration::ZERO));
        let duration_written_clone = duration_written.clone();

        let (kill_tx, kill_rx) = crossbeam_channel::bounded(1);
        let handle = std::thread::spawn(move || -> GraphRunResult<()> {
            loop {
                if kill_rx.try_recv().is_ok() {
                    return Ok(());
                }

                while output_stream.output_samples_needed() > 0 {
                    if kill_rx.try_recv().is_ok() {
                        return Ok(());
                    }

                    graph.with_inner(|graph| -> GraphRunResult<()> {
                        let block_size = output_stream.block_size();
                        if block_size != graph.block_size {
                            if block_size > graph.block_size {
                                graph.allocate(output_stream.sample_rate(), block_size);
                            } else {
                                graph.resize_buffers(output_stream.sample_rate(), block_size);
                            }
                        }

                        graph.process()?;

                        let mut delta = 0;
                        for sample_idx in 0..graph.block_size {
                            for channel_idx in 0..output_stream.output_channels() {
                                let Some(buffer) = graph.get_output(channel_idx) else {
                                    continue;
                                };

                                delta +=
                                    output_stream.output(std::iter::once(buffer[sample_idx]))?;
                            }
                        }

                        samples_written_clone.fetch_add(delta, Ordering::Relaxed);

                        let duration_secs = delta as f32
                            / output_stream.output_channels() as f32
                            / output_stream.sample_rate();
                        duration_written_clone
                            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |dur| {
                                Some(dur + Duration::from_secs_f32(duration_secs))
                            })
                            .unwrap();

                        Ok(())
                    })?;
                }
            }
        });

        Ok(RunningGraph {
            handle,
            kill_tx,
            samples_written,
            duration_written,
        })
    }

    pub fn play_for(&self, output_stream: impl AudioOut, duration: Duration) -> GraphRunResult<()> {
        let handle = self.play(output_stream)?;
        handle.run_for(duration)?;
        Ok(())
    }
}

pub struct RunningGraph {
    handle: JoinHandle<GraphRunResult<()>>,
    kill_tx: Sender<()>,
    samples_written: Arc<AtomicUsize>,
    duration_written: Arc<AtomicDuration>,
}

impl RunningGraph {
    pub fn stop(self) -> GraphRunResult<()> {
        self.kill_tx.send(()).unwrap();
        self.handle.join().unwrap()
    }

    pub fn samples_written(&self) -> usize {
        self.samples_written.load(Ordering::Relaxed)
    }

    pub fn duration_written(&self) -> Duration {
        self.duration_written.load(Ordering::Relaxed)
    }

    pub fn run_for(self, duration: Duration) -> GraphRunResult<()> {
        while self.duration_written() < duration {
            std::hint::spin_loop();
        }
        self.stop()
    }
}
