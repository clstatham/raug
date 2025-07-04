//! A directed graph of [`Processor`]s connected by [`Edge`]s.

use std::{
    ops::Deref,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    thread::JoinHandle,
    time::Duration,
};

use atomic_time::AtomicDuration;
use crossbeam_channel::Sender;
use node::{IntoNode, IntoOutputs, Node, ProcessNodeError, ProcessorNode};
use raug_graph::{
    builder::IntoIndex,
    graph::{AbstractGraph, Connection, EdgeIndex, NodeIndex},
    petgraph::{self, Direction, visit::EdgeRef},
    prelude::GraphBuilder,
};
use runtime::{AudioDevice, AudioOut};
use rustc_hash::FxHashMap;

use crate::{
    prelude::{Constant, Null, Passthrough, ProcEnv},
    processor::{Processor, io::ProcessMode},
    signal::{Signal, type_erased::AnyBuffer},
};

pub mod node;
pub mod runtime;
pub mod sub_graph;

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
    pub(crate) graph: raug_graph::graph::Graph<Self>,

    // cached strongly connected components (feedback loops)
    sccs: Vec<Vec<NodeIndex>>,

    pub(crate) sample_rate: f32,
    pub(crate) block_size: usize,
    pub(crate) max_block_size: usize,
}

pub type Edge = Connection<GraphInner>;

impl AbstractGraph for GraphInner {
    type Node = ProcessorNode;
    type Edge = ();

    fn duplicate_connection_mode() -> raug_graph::prelude::DuplicateConnectionMode {
        raug_graph::graph::DuplicateConnectionMode::Disconnect
    }

    fn graph(&self) -> &raug_graph::prelude::Graph<Self> {
        &self.graph
    }

    fn graph_mut(&mut self) -> &mut raug_graph::prelude::Graph<Self> {
        &mut self.graph
    }
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
        self.graph.add_input(ProcessorNode::new(Null::default()))
    }

    /// Adds an audio output node to the graph.
    pub(crate) fn add_audio_output(&mut self) -> NodeIndex {
        let mut node = ProcessorNode::new(Passthrough::<f32>::default());
        node.allocate(self.sample_rate, self.max_block_size);
        node.resize_buffers(self.sample_rate, self.max_block_size);
        self.graph.add_output(node)
    }

    pub fn add_node(&mut self, mut node: ProcessorNode) -> NodeIndex {
        // allocate for the node on-the-fly with the current sample rate and
        // block size, so that it can immediately be used
        node.allocate(self.sample_rate, self.max_block_size);
        node.resize_buffers(self.sample_rate, self.max_block_size);

        self.graph.add_node(node)
    }

    /// Adds a processor node to the graph.
    pub fn add_processor(&mut self, processor: impl Processor) -> NodeIndex {
        self.add_node(ProcessorNode::new(processor))
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
    ) -> EdgeIndex {
        self.graph
            .connect(source, source_output, target, target_input)
            .unwrap()
    }

    /// Disconnects two nodes in the graph at the specified input and output indices.
    ///
    /// Does nothing if the edge does not exist.
    pub fn disconnect(&mut self, target: NodeIndex, target_input: u32) {
        self.graph.disconnect(target, target_input);
    }

    /// Disconnects all inputs to the specified node.
    pub fn disconnect_all_inputs(&mut self, node: NodeIndex) -> Vec<Edge> {
        self.graph.disconnect_all_inputs(node)
    }

    /// Disconnects all outputs from the specified node.
    pub fn disconnect_all_outputs(&mut self, node: NodeIndex) -> Vec<Edge> {
        self.graph.disconnect_all_outputs(node)
    }

    /// Disconnects all inputs and outputs from the specified node.
    pub fn disconnect_all(&mut self, node: NodeIndex) -> Vec<Edge> {
        self.graph.disconnect_all(node)
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

            if source_output < self.graph[replacement].num_outputs() as u32 {
                self.connect(replacement, source_output, target, target_input);
            } else {
                // leave it disconnected
            }
        }

        // self.graph.garbage_collect();

        replacement
    }

    /// Returns the number of audio inputs in the graph.
    #[inline]
    pub fn num_audio_inputs(&self) -> usize {
        self.graph.num_inputs()
    }

    /// Returns the number of audio outputs in the graph.
    #[inline]
    pub fn num_audio_outputs(&self) -> usize {
        self.graph.num_outputs()
    }

    /// Returns the indices of the audio inputs in the graph.
    #[inline]
    pub fn input_indices(&self) -> &[NodeIndex] {
        self.graph.inputs()
    }

    /// Returns the indices of the audio outputs in the graph.
    #[inline]
    pub fn output_indices(&self) -> &[NodeIndex] {
        self.graph.outputs()
    }

    /// Returns a mutable reference to the graph's input buffer for the given input index.
    #[inline]
    pub fn get_input_mut(&mut self, input_index: usize) -> Option<&mut [f32]> {
        let input_index = *self.input_indices().get(input_index)?;
        self.graph
            .digraph_mut()
            .node_weight_mut(input_index)
            .map(|node| node.outputs[0].as_mut_slice::<f32>().unwrap())
    }

    /// Returns a reference to the graph's output buffer for the given output index.
    #[inline]
    pub fn get_output(&self, output_index: usize) -> Option<&[f32]> {
        let output_index = *self.output_indices().get(output_index)?;
        self.graph
            .digraph()
            .node_weight(output_index)
            .map(|buffers| buffers.outputs[0].as_slice::<f32>().unwrap())
    }

    #[inline]
    pub(crate) fn sccs(&self) -> &[Vec<NodeIndex>] {
        &self.sccs
    }

    #[inline]
    pub(crate) fn detect_sccs(&mut self) {
        self.sccs = petgraph::algo::kosaraju_scc(&self.graph.digraph());
        self.sccs.reverse();
    }

    pub fn reset_visitor(&mut self) {
        if self.graph.needs_visitor_reset {
            self.detect_sccs();
        }
        self.graph.reset_visitor();
    }

    /// Calls [`Processor::allocate()`] on each node in the graph.
    pub fn allocate(&mut self, sample_rate: f32, max_block_size: usize) {
        self.reset_visitor();
        self.graph.visit_mut(|_id, node| {
            node.allocate(sample_rate, max_block_size);
            raug_graph::graph::VisitResult::Continue::<()>
        });
        self.resize_buffers(sample_rate, max_block_size);

        self.sample_rate = sample_rate;
        self.block_size = max_block_size;
        self.max_block_size = max_block_size;
    }

    /// Calls [`Processor::resize_buffers()`] on each node in the graph.
    pub fn resize_buffers(&mut self, sample_rate: f32, block_size: usize) {
        self.reset_visitor();
        self.graph.visit_mut(|_id, node| {
            node.resize_buffers(sample_rate, block_size);
            raug_graph::graph::VisitResult::Continue::<()>
        });

        self.sample_rate = sample_rate;
        self.block_size = block_size;
    }

    /// Writes a DOT representation of the graph to the provided writer, suitable for rendering with Graphviz.
    pub fn write_dot<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let mut node_names = FxHashMap::default();
        let mut graph = dot_graph::Graph::new("Raug", dot_graph::Kind::Digraph);
        for (i, node) in self.graph.digraph().node_indices().enumerate() {
            let node_name = format!("_{}_{}", i, self.graph[node].name());
            graph.add_node(dot_graph::Node::new(&node_name).label(self.graph[node].name()));
            node_names.insert(node, node_name);
        }

        for edge in self.graph.digraph().edge_weights() {
            let source = &self.graph[edge.source];
            let target = &self.graph[edge.target];
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
            .graph
            .digraph()
            .edges_directed(node_id, Direction::Incoming)
            .map(|edge| (edge.source(), edge.weight()))
        {
            let source_buffers = &self.graph[source_id].outputs;
            let buffer = &source_buffers[edge.source_output as usize] as *const AnyBuffer;

            inputs[edge.target_input as usize] = Some(buffer);
        }

        let node = &mut self.graph[node_id];

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
    pub(crate) inner: GraphBuilder<GraphInner>,
}

impl Deref for Graph {
    type Target = GraphBuilder<GraphInner>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Graph {
    /// Creates a new empty `Graph`.
    pub fn new(inputs: usize, outputs: usize) -> Self {
        Self {
            inner: GraphBuilder::from_inner(GraphInner::new(inputs, outputs)),
        }
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
            let outputs = graph.output_indices().to_vec();
            for (o, i) in outputs.into_iter().zip(inputs) {
                graph.connect(i.node_id(), i.output_index, o, 0);
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
        self.with_inner(|graph| graph.graph.digraph().node_count())
    }

    /// Returns the number of edges in the graph.
    pub fn edge_count(&self) -> usize {
        self.with_inner(|graph| graph.graph.digraph().edge_count())
    }

    /// Disconnects the given output of one node from the given input of another node.
    #[track_caller]
    #[inline]
    pub fn disconnect(&self, to: impl IntoNode, to_input: impl IntoIndex) {
        let to = to.into_node(self);
        let to_input = to_input.into_input_idx(&to).expect("Invalid input index");
        self.with_inner(|graph| graph.disconnect(to.id(), to_input));
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
                        if block_size > graph.max_block_size {
                            log::debug!("Reallocating graph buffers to {} samples", block_size);
                            graph.allocate(output_stream.sample_rate(), block_size);
                        } else if block_size != graph.block_size {
                            log::debug!("Resizing graph buffers to {} samples", block_size);
                            graph.resize_buffers(output_stream.sample_rate(), block_size);
                        }

                        graph.process()?;

                        let mut delta = 0;
                        for sample_idx in 0..graph.block_size {
                            for channel_idx in 0..output_stream.output_channels() {
                                let Some(buffer) = graph.get_output(channel_idx) else {
                                    continue;
                                };

                                delta += output_stream.output(&[buffer[sample_idx]])?;
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
