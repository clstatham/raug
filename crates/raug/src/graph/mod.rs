//! A directed graph of [`Processor`]s connected by [`Connection`]s.

use std::ops::Deref;

use node::{ProcessNodeError, ProcessorNode};
use raug_graph::{
    node::{AsNodeInputIndex, AsNodeOutputIndex},
    petgraph::{self, Direction, visit::EdgeRef},
};
use rustc_hash::FxHashMap;

use crate::{
    graph::node::{BuildOnGraph, IntoNodeInput, IntoNodeOutput, RaugNodeIndexExt},
    prelude::{Constant, Null, Param, Passthrough, ProcEnv},
    processor::{Processor, io::ProcessMode},
    signal::Signal,
};

pub mod node;
#[cfg(feature = "playback")]
pub mod playback;
pub mod sub_graph;

pub use raug_graph::graph::Connection;

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

    pub fn block_size(&self) -> usize {
        self.block_size
    }

    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    pub fn has_node(&self, node: Node) -> bool {
        self.graph.digraph().node_weight(node.0).is_some()
    }

    pub fn get_node(&self, index: usize) -> Option<&ProcessorNode> {
        self.graph
            .digraph()
            .node_weight(raug_graph::graph::NodeIndex::new(index))
    }

    pub fn get_node_mut(&mut self, index: usize) -> &mut ProcessorNode {
        &mut self.graph[raug_graph::graph::NodeIndex::new(index)]
    }

    pub fn node_id_iter(&self) -> impl Iterator<Item = Node> + '_ {
        self.graph.digraph().node_indices().map(Node)
    }

    pub fn node_iter(&self) -> impl Iterator<Item = &ProcessorNode> {
        self.graph.digraph().node_weights()
    }

    pub fn edge_iter(&self) -> impl Iterator<Item = &Connection> {
        self.graph.digraph().edge_weights()
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

    pub fn processor_boxed(&mut self, proc: Box<dyn Processor>) -> Node {
        let mut node = ProcessorNode::new_boxed(proc);

        // allocate for the node on-the-fly with the current sample rate and
        // block size, so that it can immediately be used
        node.allocate(self.sample_rate, self.max_block_size);
        node.resize_buffers(self.sample_rate, self.max_block_size);

        Node(self.graph.add_node(node))
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

    pub fn param<T>(&mut self, value: T) -> Node
    where
        T: Signal + Copy + Default,
    {
        self.node(Param::new(value))
    }

    /// Connects two nodes in the graph.
    pub fn connect<O, Src, I, Tgt>(&mut self, source: Src, target: Tgt)
    where
        O: AsNodeOutputIndex<ProcessorNode>,
        I: AsNodeInputIndex<ProcessorNode>,
        Src: IntoNodeOutput<O>,
        Tgt: IntoNodeInput<I>,
    {
        let source = source.into_node_output(self);
        let target = target.into_node_input(self);
        self.graph.connect(*source, *target).unwrap();
    }

    /// Connects a constant value to the specified target input.
    ///
    /// The constant processor is created and added to the graph automatically.
    pub fn connect_constant<T, I, Tgt>(&mut self, value: T, target: Tgt)
    where
        I: AsNodeInputIndex<ProcessorNode>,
        Tgt: IntoNodeInput<I>,
        T: Signal + Clone,
    {
        let constant = self.constant(value);
        self.connect(constant, target);
    }

    /// Connects a parameter with the specified initial value to the specified target input.
    ///
    /// The parameter processor is created and added to the graph automatically.
    ///
    /// Returns a linked clone of the created `Param<T>` instance, which can be used to update the parameter value at runtime.
    pub fn connect_param<T, I, Tgt>(&mut self, init: T, target: Tgt) -> Param<T>
    where
        I: AsNodeInputIndex<ProcessorNode>,
        Tgt: IntoNodeInput<I>,
        T: Signal + Copy + Default,
    {
        let param = Param::new(init);
        let node = self.node(param.clone());
        self.connect(node.output(0), target);
        param
    }

    /// Creates an audio input node and connects it to the specified target input.
    pub fn connect_audio_input<I, Tgt>(&mut self, target: Tgt)
    where
        I: AsNodeInputIndex<ProcessorNode>,
        Tgt: IntoNodeInput<I>,
    {
        let input = self.audio_input();
        self.connect(input.output(0), target);
    }

    /// Creates an audio output node and connects the specified source to its input.
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
    pub fn disconnect<O, Src, I, Tgt>(&mut self, source: Src, target: Tgt)
    where
        O: AsNodeOutputIndex<ProcessorNode>,
        I: AsNodeInputIndex<ProcessorNode>,
        Src: IntoNodeOutput<O>,
        Tgt: IntoNodeInput<I>,
    {
        let source = source.into_node_output(self);
        let target = target.into_node_input(self);
        self.graph.disconnect(*source, *target);
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
        let block_size = if block_size > self.max_block_size {
            log::warn!(
                "Requested block size {} exceeds max block size {}, resizing to max block size",
                block_size,
                self.max_block_size
            );
            self.max_block_size
        } else {
            block_size
        };
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
    pub fn process(&mut self) -> Result<(), ProcessNodeError> {
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

    fn process_node(&mut self, node_id: Node, mode: ProcessMode) -> Result<(), ProcessNodeError> {
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
    pub fn constant<T: Signal + Clone>(&mut self, value: T) -> Node {
        self.node(Constant::new(value))
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
