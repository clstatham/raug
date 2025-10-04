//! A directed graph of [`Processor`]s connected by [`Connection`]s.

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
use crossbeam_channel::{Receiver, Sender};
use node::{ProcessNodeError, ProcessorNode};
use raug_graph::{
    graph::Connection,
    node::{AsNodeInputIndex, AsNodeOutputIndex, NodeInput},
    petgraph::{self, Direction, visit::EdgeRef},
};
use runtime::{AudioDevice, AudioOut};
use rustc_hash::FxHashMap;

use crate::{
    graph::node::{BuildOnGraph, Input, IntoNodeOutput, Output, RaugNodeIndexExt},
    prelude::{Constant, Null, Passthrough, ProcEnv},
    processor::{Processor, io::ProcessMode},
    signal::Signal,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct Node(raug_graph::graph::NodeIndex);

impl Node {
    pub fn new(index: usize) -> Self {
        Node(raug_graph::graph::NodeIndex::new(index))
    }
}

impl Deref for Node {
    type Target = raug_graph::graph::NodeIndex;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<raug_graph::graph::NodeIndex> for Node {
    fn from(value: raug_graph::graph::NodeIndex) -> Self {
        Node(value)
    }
}

impl From<Node> for raug_graph::graph::NodeIndex {
    fn from(value: Node) -> raug_graph::graph::NodeIndex {
        value.0
    }
}

#[derive(Default)]
pub struct Graph {
    pub(crate) graph: raug_graph::graph::Graph<ProcessorNode>,

    // cached strongly connected components (feedback loops)
    sccs: Vec<Vec<Node>>,

    pub(crate) sample_rate: f32,
    pub(crate) block_size: usize,
    pub(crate) max_block_size: usize,
}

impl Graph {
    /// Creates a new empty `Graph`.
    pub fn new() -> Self {
        Self::default()
    }

    pub fn node_count(&self) -> usize {
        self.graph.digraph().node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.graph.digraph().edge_count()
    }

    pub fn estimate_ram_usage(&self) -> usize {
        let mut total = 0;
        for node in self.graph.digraph().node_weights() {
            for output in &node.outputs {
                total += output.len() * std::mem::size_of::<f32>();
            }
        }
        total
    }

    /// Adds an audio input node to the graph.
    pub fn audio_input(&mut self) -> Node {
        Node(self.graph.add_input(ProcessorNode::new(Null::default())))
    }

    /// Adds an audio output node to the graph.
    pub fn audio_output(&mut self) -> Node {
        let mut node = ProcessorNode::new(Passthrough::<f32>::default());
        node.allocate(self.sample_rate, self.max_block_size);
        node.resize_buffers(self.sample_rate, self.max_block_size);
        Node(self.graph.add_output(node))
    }

    pub fn processor(&mut self, node: impl Processor) -> Node {
        let mut node = ProcessorNode::new(node);
        // allocate for the node on-the-fly with the current sample rate and
        // block size, so that it can immediately be used
        node.allocate(self.sample_rate, self.max_block_size);
        node.resize_buffers(self.sample_rate, self.max_block_size);

        Node(self.graph.add_node(node))
    }

    pub fn node(&mut self, node: impl BuildOnGraph) -> Node {
        node.build_on_graph(self)
    }

    /// Connects two nodes in the graph.
    ///
    /// If the edge already exists, this function does nothing.
    ///
    /// If the target node already has an incoming edge at the target input, the existing edge is removed.
    pub fn connect<O, Src, I, Tgt>(&mut self, source: Src, target: Tgt)
    where
        O: AsNodeOutputIndex<ProcessorNode>,
        I: AsNodeInputIndex<ProcessorNode>,
        Src: IntoNodeOutput<O>,
        Tgt: Into<Input<I>> + Copy,
    {
        let source = source.into_node_output(self);
        self.graph.connect(*source, *target.into()).unwrap();
    }

    pub fn connect_constant<I, Tgt>(&mut self, value: f32, target: Tgt)
    where
        I: AsNodeInputIndex<ProcessorNode>,
        Tgt: Into<Input<I>> + Copy,
    {
        let constant = self.constant(value);
        self.connect(constant, target);
    }

    pub fn connect_audio_output<O, Src>(&mut self, source: Src)
    where
        O: AsNodeOutputIndex<ProcessorNode>,
        Src: IntoNodeOutput<O>,
    {
        let output = self.audio_output();
        self.connect(source, output.input(0));
    }

    pub fn bin_op<A, B, Op, O1, O2>(&mut self, a: A, op: Op, b: B) -> Node
    where
        Op: Processor + Default,
        O1: AsNodeOutputIndex<ProcessorNode>,
        O2: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<O1>,
        B: IntoNodeOutput<O2>,
    {
        let node = self.node(op);
        self.connect(a, node.input(0));
        self.connect(b, node.input(1));
        node
    }

    pub fn un_op<A, Op, O>(&mut self, op: Op, a: A) -> Node
    where
        Op: Processor + Default,
        O: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<O>,
    {
        let node = self.node(op);
        self.connect(a, node.input(0));
        node
    }

    pub fn add<A, AO, B, BO>(&mut self, a: A, b: B) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        BO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>,
        B: IntoNodeOutput<BO>,
    {
        self.bin_op(a, crate::builtins::Add::default(), b)
    }

    pub fn sub<A, AO, B, BO>(&mut self, a: A, b: B) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        BO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>,
        B: IntoNodeOutput<BO>,
    {
        self.bin_op(a, crate::builtins::Sub::default(), b)
    }

    pub fn mul<A, AO, B, BO>(&mut self, a: A, b: B) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        BO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>,
        B: IntoNodeOutput<BO>,
    {
        self.bin_op(a, crate::builtins::Mul::default(), b)
    }

    pub fn div<A, AO, B, BO>(&mut self, a: A, b: B) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        BO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>,
        B: IntoNodeOutput<BO>,
    {
        self.bin_op(a, crate::builtins::Div::default(), b)
    }

    pub fn rem<A, AO, B, BO>(&mut self, a: A, b: B) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        BO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>,
        B: IntoNodeOutput<BO>,
    {
        self.bin_op(a, crate::builtins::Rem::default(), b)
    }

    /// Disconnects two nodes in the graph at the specified input and output indices.
    ///
    /// Does nothing if the edge does not exist.
    pub fn disconnect<I, Tgt>(&mut self, target: Tgt) -> Option<Connection>
    where
        I: AsNodeInputIndex<ProcessorNode>,
        Tgt: Into<NodeInput<ProcessorNode, I>> + Copy,
    {
        self.graph.disconnect(target)
    }

    /// Disconnects all inputs to the specified node.
    pub fn disconnect_all_inputs(&mut self, node: Node) -> Vec<Connection> {
        self.graph.disconnect_all_inputs(node.into())
    }

    /// Disconnects all outputs from the specified node.
    pub fn disconnect_all_outputs(&mut self, node: Node) -> Vec<Connection> {
        self.graph.disconnect_all_outputs(node.into())
    }

    /// Disconnects all inputs and outputs from the specified node.
    pub fn disconnect_all(&mut self, node: Node) -> Vec<Connection> {
        self.graph.disconnect_all(node.into())
    }

    pub fn replace_node_gracefully(&mut self, replaced: Node, replacement: Node) -> Node {
        let replaced_outputs = self.disconnect_all_outputs(replaced);

        for edge in replaced_outputs {
            let Connection {
                target,
                source_output,
                target_input,
                ..
            } = edge;

            let target = Node(target);

            if source_output < self.graph[replacement.0].num_outputs() as u32 {
                self.connect(
                    replacement.output(source_output),
                    target.input(target_input),
                );
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
    pub fn input_indices(&self) -> impl Iterator<Item = Node> + '_ {
        self.graph.inputs().iter().copied().map(Node)
    }

    /// Returns the indices of the audio outputs in the graph.
    #[inline]
    pub fn output_indices(&self) -> impl Iterator<Item = Node> + '_ {
        self.graph.outputs().iter().copied().map(Node)
    }

    /// Returns a mutable reference to the graph's input buffer for the given input index.
    #[inline]
    pub fn get_input_mut(&mut self, input_index: usize) -> Option<&mut [f32]> {
        let input_index = *self.input_indices().nth(input_index)?;
        self.graph
            .digraph_mut()
            .node_weight_mut(input_index)
            .map(|node| node.outputs[0].as_mut_slice::<f32>().unwrap())
    }

    /// Returns a reference to the graph's output buffer for the given output index.
    #[inline]
    pub fn get_output(&self, output_index: usize) -> Option<&[f32]> {
        let output_index = *self.output_indices().nth(output_index)?;
        self.graph
            .digraph()
            .node_weight(output_index)
            .map(|buffers| buffers.outputs[0].as_slice::<f32>().unwrap())
    }

    #[inline]
    pub(crate) fn sccs(&self) -> &[Vec<Node>] {
        &self.sccs
    }

    #[inline]
    pub(crate) fn detect_sccs(&mut self) {
        let sccs = petgraph::algo::kosaraju_scc(&self.graph.digraph());
        self.sccs.clear();
        for scc in sccs {
            if !scc.is_empty() {
                self.sccs.push(scc.into_iter().map(Node).collect());
            }
        }
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

    fn process_node(&mut self, node_id: Node, mode: ProcessMode) -> GraphRunResult<()> {
        let num_inputs = self.graph[node_id.0].num_inputs();
        let mut inputs: smallvec::SmallVec<[_; 32]> = smallvec::smallvec![None; num_inputs];

        for (source_id, edge) in self
            .graph
            .digraph()
            .edges_directed(node_id.0, Direction::Incoming)
            .map(|edge| (edge.source(), *edge.weight()))
        {
            if source_id == node_id.0 {
                log::warn!(
                    "Self-loops are not supported, skipping edge from node {:?} ({}) to itself",
                    node_id,
                    self.graph[node_id.0].name()
                );
                continue;
            }
            // SAFETY: We've validated that the source node and the node we're processing do not overlap,
            // so no mutable aliasing can occur.
            let graph = &raw const self.graph;
            let source_buffers = unsafe { &(&*graph)[source_id].outputs };
            let buffer = &source_buffers[edge.source_output as usize];

            inputs[edge.target_input as usize] = Some(buffer);
        }

        let node = &mut self.graph[node_id.0];

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

    /// Creates a new [`Node`] that outputs a constant value.
    pub fn constant<T: Signal + Default + Clone>(&mut self, value: T) -> Node {
        self.node(Constant::new(value))
    }

    pub fn play(mut self, mut output_stream: impl AudioOut) -> GraphRunResult<RunningGraph> {
        let status = Arc::new(GraphStatus::new(self.sample_rate as usize, self.block_size));
        let status_clone = status.clone();

        let (kill_tx, kill_rx) = crossbeam_channel::bounded(1);
        let (request_tx, request_rx) = crossbeam_channel::unbounded();
        let (response_tx, response_rx) = crossbeam_channel::unbounded();
        let handle = std::thread::spawn(move || -> GraphRunResult<Graph> {
            loop {
                if kill_rx.try_recv().is_ok() {
                    return Ok(self);
                }

                while let Ok(request) = request_rx.try_recv() {
                    match request {
                        GraphRequest::AddNode(node) => {
                            let node = self.graph.add_node(node);
                            response_tx
                                .send(GraphResponse::NodeAdded(Node(node)))
                                .map_err(|_| GraphRunError::StreamSendError)?;
                        }
                        GraphRequest::RemoveNode(node) => {
                            self.disconnect_all(node);
                            response_tx
                                .send(GraphResponse::NodeRemoved(node))
                                .map_err(|_| GraphRunError::StreamSendError)?;
                        }
                        GraphRequest::ConnectU32U32 {
                            source,
                            source_output,
                            target,
                            target_input,
                        } => {
                            if source_output < self.graph[source.0].num_outputs() as u32
                                && target_input < self.graph[target.0].num_inputs() as u32
                            {
                                self.connect(
                                    source.output(source_output),
                                    target.input(target_input),
                                );
                                response_tx
                                    .send(GraphResponse::Connected)
                                    .map_err(|_| GraphRunError::StreamSendError)?;
                            } else {
                                return Err(GraphRunError::GraphModificationError(
                                    GraphConstructionError::InputIndexOutOfBounds {
                                        index: target_input as usize,
                                        num_inputs: self.graph[target.0].num_inputs(),
                                    },
                                ));
                            }
                        }
                    }
                }

                while output_stream.output_samples_needed() > 0 {
                    if kill_rx.try_recv().is_ok() {
                        return Ok(self);
                    }

                    let block_size = output_stream.block_size();
                    if block_size > self.max_block_size {
                        log::debug!("Reallocating graph buffers to {} samples", block_size);
                        self.allocate(output_stream.sample_rate(), block_size);
                        status_clone.block_size.store(block_size, Ordering::Relaxed);
                    } else if block_size != self.block_size {
                        log::debug!("Resizing graph buffers to {} samples", block_size);
                        self.resize_buffers(output_stream.sample_rate(), block_size);
                        status_clone.block_size.store(block_size, Ordering::Relaxed);
                    }

                    status_clone
                        .sample_rate
                        .store(output_stream.sample_rate() as usize, Ordering::Relaxed);

                    self.process()?;

                    let mut delta = 0;
                    for sample_idx in 0..self.block_size {
                        for channel_idx in 0..output_stream.output_channels() {
                            let Some(buffer) = self.get_output(channel_idx) else {
                                continue;
                            };

                            delta += output_stream.write(&[buffer[sample_idx]])?;
                        }
                    }

                    status_clone
                        .samples_written
                        .fetch_add(delta, Ordering::Relaxed);

                    let duration_secs = delta as f32
                        / output_stream.output_channels() as f32
                        / output_stream.sample_rate();
                    status_clone
                        .duration_written
                        .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |dur| {
                            Some(dur + Duration::from_secs_f32(duration_secs))
                        })
                        .unwrap();
                }
            }
        });

        Ok(RunningGraph {
            handle,
            kill_tx,
            request_tx,
            response_rx,
            status,
        })
    }

    pub fn play_for(
        self,
        output_stream: impl AudioOut,
        duration: Duration,
    ) -> GraphRunResult<Graph> {
        let handle = self.play(output_stream)?;
        let graph = handle.run_for(duration)?;
        Ok(graph)
    }
}

impl std::ops::Index<Node> for Graph {
    type Output = ProcessorNode;

    fn index(&self, index: Node) -> &Self::Output {
        &self.graph[index.0]
    }
}

impl std::ops::IndexMut<Node> for Graph {
    fn index_mut(&mut self, index: Node) -> &mut Self::Output {
        &mut self.graph[index.0]
    }
}

pub struct GraphStatus {
    samples_written: AtomicUsize,
    duration_written: AtomicDuration,
    sample_rate: AtomicUsize,
    block_size: AtomicUsize,
}

impl GraphStatus {
    fn new(sample_rate: usize, block_size: usize) -> Self {
        Self {
            samples_written: AtomicUsize::new(0),
            duration_written: AtomicDuration::new(Duration::ZERO),
            sample_rate: AtomicUsize::new(sample_rate),
            block_size: AtomicUsize::new(block_size),
        }
    }
}

pub enum GraphRequest {
    AddNode(ProcessorNode),
    RemoveNode(Node),
    ConnectU32U32 {
        source: Node,
        source_output: u32,
        target: Node,
        target_input: u32,
    },
}

#[derive(Debug)]
pub enum GraphResponse {
    NodeAdded(Node),
    NodeRemoved(Node),
    Connected,
}

pub struct RunningGraph {
    handle: JoinHandle<GraphRunResult<Graph>>,
    kill_tx: Sender<()>,
    request_tx: Sender<GraphRequest>,
    response_rx: Receiver<GraphResponse>,
    status: Arc<GraphStatus>,
}

impl RunningGraph {
    pub fn stop(self) -> GraphRunResult<Graph> {
        self.kill_tx.send(()).unwrap();
        self.handle.join().unwrap()
    }

    pub fn samples_written(&self) -> usize {
        self.status.samples_written.load(Ordering::Relaxed)
    }

    pub fn duration_written(&self) -> Duration {
        self.status.duration_written.load(Ordering::Relaxed)
    }

    pub fn sample_rate(&self) -> usize {
        self.status.sample_rate.load(Ordering::Relaxed)
    }

    pub fn block_size(&self) -> usize {
        self.status.block_size.load(Ordering::Relaxed)
    }

    pub fn run_for(self, duration: Duration) -> GraphRunResult<Graph> {
        while self.duration_written() < duration {
            std::thread::sleep(Duration::from_millis(10));
        }
        self.stop()
    }

    pub fn node(&self, node: impl Processor) -> GraphRunResult<Node> {
        let mut node = ProcessorNode::new(node);
        node.allocate(self.sample_rate() as f32, self.block_size());
        node.resize_buffers(self.sample_rate() as f32, self.block_size());
        match self.request_tx.send(GraphRequest::AddNode(node)) {
            Ok(()) => {}
            Err(_) => return Err(GraphRunError::StreamNotSpawned),
        };

        match self.response_rx.recv() {
            Ok(GraphResponse::NodeAdded(node)) => Ok(node),
            Ok(_) => Err(GraphRunError::StreamReceiveError),
            Err(_) => Err(GraphRunError::StreamReceiveError),
        }
    }

    pub fn remove_node(&self, node: Node) -> GraphRunResult<Node> {
        match self.request_tx.send(GraphRequest::RemoveNode(node)) {
            Ok(()) => {}
            Err(_) => return Err(GraphRunError::StreamNotSpawned),
        };

        match self.response_rx.recv() {
            Ok(GraphResponse::NodeRemoved(node)) => Ok(node),
            Ok(_) => Err(GraphRunError::StreamReceiveError),
            Err(_) => Err(GraphRunError::StreamReceiveError),
        }
    }

    pub fn connect(&self, source: Output<u32>, target: Input<u32>) -> GraphRunResult<()> {
        match self.request_tx.send(GraphRequest::ConnectU32U32 {
            source: source.node.into(),
            source_output: source.index,
            target: target.node.into(),
            target_input: target.index,
        }) {
            Ok(()) => {}
            Err(_) => return Err(GraphRunError::StreamNotSpawned),
        };

        match self.response_rx.recv() {
            Ok(GraphResponse::Connected) => Ok(()),
            Ok(_) => Err(GraphRunError::StreamReceiveError),
            Err(_) => Err(GraphRunError::StreamReceiveError),
        }
    }
}
