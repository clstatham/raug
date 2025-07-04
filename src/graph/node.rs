//! Contains the [`ProcessorNode`] and [`Node`] structs, which represent nodes in the audio graph that process signals.

use std::{fmt::Debug, ops::Deref, sync::Arc};

use parking_lot::Mutex;
use raug_graph::{
    builder::{IntoIndex, IntoInput, IntoOutput},
    node::AbstractNode,
    prelude::{GraphBuilder, NodeBuilder},
};
use thiserror::Error;

use crate::{
    prelude::*,
    signal::{SignalType, type_erased::AnyBuffer},
};

use super::{Graph, GraphInner, NodeIndex};

/// Error that can occur during the processing of a node.
#[derive(Error, Debug)]
#[error("Error processing node '{node_name}': {error})")]
pub struct ProcessNodeError {
    pub error: ProcessorError,
    pub node_name: String,
}

impl ProcessNodeError {
    pub fn node_name(&self) -> &str {
        &self.node_name
    }

    pub fn error(&self) -> &ProcessorError {
        &self.error
    }
}

/// A node in the audio graph that processes signals.
pub struct ProcessorNode {
    pub(crate) processor: Arc<Mutex<dyn Processor>>,
    pub(crate) name: String,
    pub(crate) input_spec: Vec<SignalSpec>,
    pub(crate) output_spec: Vec<SignalSpec>,
    pub(crate) outputs: Vec<AnyBuffer>,
}

impl Debug for ProcessorNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}

impl AbstractNode for ProcessorNode {
    fn num_inputs(&self) -> usize {
        self.input_spec.len()
    }

    fn num_outputs(&self) -> usize {
        self.output_spec.len()
    }

    fn input_type(&self, index: u32) -> Option<raug_graph::TypeInfo> {
        self.input_spec
            .get(index as usize)
            .map(|v| v.signal_type.into())
    }

    fn output_type(&self, index: u32) -> Option<raug_graph::TypeInfo> {
        self.output_spec
            .get(index as usize)
            .map(|v| v.signal_type.into())
    }

    fn input_name(&self, index: u32) -> Option<&str> {
        self.input_spec.get(index as usize).map(|v| v.name.as_str())
    }

    fn output_name(&self, index: u32) -> Option<&str> {
        self.output_spec
            .get(index as usize)
            .map(|v| v.name.as_str())
    }
}

impl ProcessorNode {
    /// Creates a new `ProcessorNode` with the given processor.
    pub fn new(processor: impl Processor) -> Self {
        let name = processor.name().to_string();
        let input_spec = processor.input_spec();
        let output_spec = processor.output_spec();
        let outputs = processor.create_output_buffers(0);
        Self {
            processor: Arc::new(Mutex::new(processor)),
            name,
            input_spec,
            output_spec,
            outputs,
        }
    }

    /// Returns the name of the processor.
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns information about the input signals of the processor.
    #[inline]
    pub fn input_spec(&self) -> &[SignalSpec] {
        &self.input_spec
    }

    /// Returns information about the output signals of the processor.
    #[inline]
    pub fn output_spec(&self) -> &[SignalSpec] {
        &self.output_spec
    }

    /// Returns the number of input signals of the processor.
    #[inline]
    pub fn num_inputs(&self) -> usize {
        self.input_spec.len()
    }

    /// Returns the number of output signals of the processor.
    #[inline]
    pub fn num_outputs(&self) -> usize {
        self.output_spec.len()
    }

    /// Allocates memory for the processor.
    #[inline]
    pub fn allocate(&mut self, sample_rate: f32, max_block_size: usize) {
        let mut processor = self.processor.lock();
        processor.allocate(sample_rate, max_block_size);
        self.outputs = processor.create_output_buffers(max_block_size);
    }

    /// Resizes the internal buffers of the processor and updates the sample rate and block size.
    ///
    /// This function is NOT ALLOWED to allocate memory.
    #[inline]
    pub fn resize_buffers(&mut self, sample_rate: f32, block_size: usize) {
        self.processor
            .lock()
            .resize_buffers(sample_rate, block_size);
    }

    /// Processes the input signals and writes the output signals to the given buffers.
    #[inline]
    pub(crate) fn process(
        &mut self,
        inputs: &[Option<*const AnyBuffer>],
        env: ProcEnv,
    ) -> Result<(), ProcessNodeError> {
        let inputs = ProcessorInputs {
            input_specs: &self.input_spec,
            inputs,
            env,
        };
        let outputs = ProcessorOutputs {
            output_spec: &self.output_spec,
            outputs: &mut self.outputs,
            mode: env.mode,
        };
        if let Err(e) = self.processor.lock().process(inputs, outputs) {
            return Err(ProcessNodeError {
                error: e,
                node_name: self.name().to_string(),
            });
        }

        Ok(())
    }
}

/// Represents a node in the audio graph. This type is used to build connections between nodes.
#[derive(Clone)]
pub struct Node {
    inner: NodeBuilder<GraphInner>,
}

impl Deref for Node {
    type Target = NodeBuilder<GraphInner>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name().as_str())
    }
}

impl IntoOutput<GraphInner> for Node {
    fn into_output(
        self,
        graph: &GraphBuilder<GraphInner>,
    ) -> std::result::Result<raug_graph::builder::Output<GraphInner>, raug_graph::Error> {
        assert!(
            graph.is_same_graph(&self.inner.graph()),
            "Node must belong to the same graph"
        );
        self.inner.output(0)
    }
}

impl IntoOutput<GraphInner> for &Node {
    fn into_output(
        self,
        graph: &GraphBuilder<GraphInner>,
    ) -> std::result::Result<raug_graph::builder::Output<GraphInner>, raug_graph::Error> {
        IntoOutput::into_output(self.clone(), graph)
    }
}

impl Node {
    pub(crate) fn new(graph: Graph, node_id: NodeIndex) -> Self {
        Node {
            inner: NodeBuilder::new(graph.inner, node_id),
        }
    }

    /// Returns the graph builder that this node belongs to.
    #[inline]
    pub fn graph(&self) -> Graph {
        Graph {
            inner: self.inner.graph(),
        }
    }

    /// Returns the name of the processor this node represents.
    #[inline]
    pub fn name(&self) -> String {
        self.inner.name().expect("Processor should have a name")
    }

    /// Returns the name of the input at the given index.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    #[inline]
    #[track_caller]
    pub fn input_name(&self, index: impl IntoIndex) -> String {
        self.inner
            .input_name(index)
            .expect("Input should have a name")
    }

    /// Returns the name of the output at the given index.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    #[inline]
    #[track_caller]
    pub fn output_name(&self, index: impl IntoIndex) -> String {
        self.inner
            .output_name(index)
            .expect("Output should have a name")
    }

    /// Returns the input of the node at the given index.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    #[inline]
    #[track_caller]
    pub fn input(&self, index: impl IntoIndex) -> Input {
        Input {
            inner: self.inner.input(index).expect("Input should exist"),
        }
    }

    /// Returns the output of the node at the given index.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    #[inline]
    pub fn output(&self, index: impl IntoIndex) -> Output {
        Output {
            inner: self.inner.output(index).expect("Output should exist"),
        }
    }

    /// Returns the signal type of the input at the given index.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    #[inline]
    #[track_caller]
    pub fn input_type(&self, index: impl IntoIndex) -> SignalType {
        self.inner
            .input_type(index)
            .expect("Input should have a signal type")
            .into()
    }

    /// Returns the signal type of the output at the given index.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    #[inline]
    #[track_caller]
    pub fn output_type(&self, index: impl IntoIndex) -> SignalType {
        self.inner
            .output_type(index)
            .expect("Output should have a signal type")
            .into()
    }
}

#[derive(Clone, Debug)]
pub struct Edge {
    pub(crate) source: Node,
    pub(crate) source_output: u32,
    pub(crate) target: Node,
    pub(crate) target_input: u32,
}

impl Edge {
    /// Returns the source node of the edge.
    #[inline]
    pub fn source(&self) -> Node {
        self.source.clone()
    }

    /// Returns the output index of the source node.
    #[inline]
    pub fn source_output(&self) -> u32 {
        self.source_output
    }

    /// Returns the target node of the edge.
    #[inline]
    pub fn target(&self) -> Node {
        self.target.clone()
    }

    /// Returns the input index of the target node.
    #[inline]
    pub fn target_input(&self) -> u32 {
        self.target_input
    }

    /// Returns the signal type of the edge.
    #[inline]
    pub fn signal_type(&self) -> SignalType {
        self.source.output_type(self.source_output)
    }

    /// Disconnects the edge from the graph.
    #[inline]
    pub fn disconnect(self) {
        let graph = self.source.graph().clone();
        graph.disconnect(self.target, self.target_input);
    }
}

/// Represents an input of a [`Node`].
#[derive(Clone)]
pub struct Input {
    pub(crate) inner: raug_graph::builder::Input<GraphInner>,
}

impl Input {
    /// Returns the signal type of the input.
    #[inline]
    pub fn signal_type(&self) -> SignalType {
        self.inner
            .type_info()
            .expect("Input should have a signal type")
            .into()
    }

    /// Returns the [`NodeIndex`] of the node that this input belongs to.
    #[inline]
    pub fn node_id(&self) -> NodeIndex {
        self.inner.node_id()
    }

    /// Returns the [`Node`] that this input belongs to.
    #[inline]
    pub fn node(&self) -> Node {
        Node {
            inner: self.inner.node(),
        }
    }

    /// Returns the [`Graph`] that this input's node is a part of.
    #[inline]
    pub fn graph(&self) -> Graph {
        self.node().graph()
    }

    /// Returns the name of the input.
    #[inline]
    pub fn name(&self) -> String {
        self.inner.name().expect("Input should have a name")
    }

    /// Connects the input to the output of another node.
    ///
    /// # Panics
    ///
    /// Panics if the output and input signals do not have the same type.
    #[inline]
    #[track_caller]
    pub fn connect(&self, output: impl IntoOutputExt) {
        let output = self::IntoOutputExt::into_output(output, &self.graph());
        self.inner.connect(output).expect("Failed to connect input");
    }
}

impl IntoInput<GraphInner> for Input {
    fn into_input(
        self,
        graph: &GraphBuilder<GraphInner>,
    ) -> std::result::Result<raug_graph::builder::Input<GraphInner>, raug_graph::Error> {
        self.inner.into_input(graph)
    }
}

macro_rules! specific_binary_op_impl {
    ($self:ident, $b:ident, $op:ident => $type:ident) => {{
        let graph = $self.graph();
        assert_eq!(
            $self.signal_type(),
            $type::signal_type(),
            "Signal type must be {} for this operation",
            stringify!($type),
        );
        let b = self::IntoOutputExt::into_output($b, &graph);
        assert_eq!(
            $self.signal_type(),
            b.signal_type(),
            "Signal types must match for this operation",
        );
        let node = graph.node($op::default());
        node.input(0).connect($self);
        node.input(1).connect(b);
        node
    }};
}

/// Represents an output of a [`Node`].
#[derive(Clone)]
pub struct Output {
    pub(crate) inner: raug_graph::builder::Output<GraphInner>,
}

impl Deref for Output {
    type Target = raug_graph::builder::Output<GraphInner>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl raug_graph::builder::IntoOutput<GraphInner> for Output {
    fn into_output(
        self,
        graph: &GraphBuilder<GraphInner>,
    ) -> std::result::Result<raug_graph::builder::Output<GraphInner>, raug_graph::Error> {
        self.inner.into_output(graph)
    }
}

impl raug_graph::builder::IntoOutput<GraphInner> for &Output {
    fn into_output(
        self,
        graph: &GraphBuilder<GraphInner>,
    ) -> std::result::Result<raug_graph::builder::Output<GraphInner>, raug_graph::Error> {
        self.inner.clone().into_output(graph)
    }
}

impl Output {
    /// Returns the [`Node`] that this output belongs to.
    #[inline]
    pub fn node(&self) -> Node {
        Node {
            inner: self.inner.node.clone(),
        }
    }

    /// Returns the [`Graph`] that this output's node is a part of.
    #[inline]
    pub fn graph(&self) -> Graph {
        self.node().graph()
    }

    /// Returns the index of the output.
    #[inline]
    pub fn index(&self) -> u32 {
        self.output_index
    }

    /// Returns the signal type of the output.
    #[inline]
    pub fn signal_type(&self) -> SignalType {
        self.inner
            .type_info()
            .expect("Output should have a signal type")
            .into()
    }

    /// Returns the name of the output.
    #[inline]
    pub fn name(&self) -> String {
        self.inner.name().expect("Output should have a name")
    }

    /// Connects the output to the input of another node.
    ///
    /// # Panics
    ///
    /// Panics if the output and input signals do not have the same type.
    #[inline]
    #[track_caller]
    pub fn connect(&self, input: &Input) {
        input.connect(self);
    }

    /// Attaches an addition processor to the nodes.
    #[inline]
    pub fn add(&self, b: impl IntoOutputExt) -> Node {
        specific_binary_op_impl!(self, b, Add => f32)
    }

    /// Attaches a subtraction processor to the nodes.
    #[inline]
    pub fn sub(&self, b: impl IntoOutputExt) -> Node {
        specific_binary_op_impl!(self, b, Sub => f32)
    }

    /// Attaches a multiplication processor to the nodes.
    #[inline]
    pub fn mul(&self, b: impl IntoOutputExt) -> Node {
        specific_binary_op_impl!(self, b, Mul => f32)
    }

    /// Attaches a division processor to the nodes.
    #[inline]
    pub fn div(&self, b: impl IntoOutputExt) -> Node {
        specific_binary_op_impl!(self, b, Div => f32)
    }

    /// Attaches a remainder processor to the nodes.
    #[inline]
    pub fn rem(&self, b: impl IntoOutputExt) -> Node {
        specific_binary_op_impl!(self, b, Rem => f32)
    }

    /// Attaches a negation processor to the node.
    #[inline]
    pub fn neg(&self) -> Node {
        let graph = self.graph();
        assert_eq!(
            self.signal_type(),
            f32::signal_type(),
            "Signal type must be f32 for this operation"
        );
        let node = graph.node(Neg::default());
        node.input(0).connect(self);
        node
    }
}

impl<T: IntoOutputExt> std::ops::Add<T> for Output {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn add(self, rhs: T) -> Self::Output {
        specific_binary_op_impl!(self, rhs, Add => f32)
    }
}

impl<T: IntoOutputExt> std::ops::Add<T> for &Output {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn add(self, rhs: T) -> Self::Output {
        self.clone() + rhs
    }
}

impl<T: IntoOutputExt> std::ops::Sub<T> for Output {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn sub(self, rhs: T) -> Self::Output {
        specific_binary_op_impl!(self, rhs, Sub => f32)
    }
}

impl<T: IntoOutputExt> std::ops::Sub<T> for &Output {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn sub(self, rhs: T) -> Self::Output {
        self.clone() - rhs
    }
}

impl<T: IntoOutputExt> std::ops::Mul<T> for Output {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn mul(self, rhs: T) -> Self::Output {
        specific_binary_op_impl!(self, rhs, Mul => f32)
    }
}

impl<T: IntoOutputExt> std::ops::Mul<T> for &Output {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn mul(self, rhs: T) -> Self::Output {
        self.clone() * rhs
    }
}

impl<T: IntoOutputExt> std::ops::Div<T> for Output {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn div(self, rhs: T) -> Self::Output {
        specific_binary_op_impl!(self, rhs, Div => f32)
    }
}

impl<T: IntoOutputExt> std::ops::Div<T> for &Output {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn div(self, rhs: T) -> Self::Output {
        self.clone() / rhs
    }
}

impl<T: IntoOutputExt> std::ops::Rem<T> for Output {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn rem(self, rhs: T) -> Self::Output {
        specific_binary_op_impl!(self, rhs, Rem => f32)
    }
}

impl<T: IntoOutputExt> std::ops::Rem<T> for &Output {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn rem(self, rhs: T) -> Self::Output {
        self.clone() % rhs
    }
}

impl std::ops::Neg for Output {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn neg(self) -> Self::Output {
        let graph = self.graph();
        assert_eq!(
            self.signal_type(),
            f32::signal_type(),
            "Signal type must be f32 for this operation"
        );
        let node = graph.node(Neg::default());
        node.input(0).connect(self);
        node
    }
}

impl std::ops::Neg for &Output {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn neg(self) -> Self::Output {
        -self.clone()
    }
}

impl<T: IntoOutputExt> std::ops::BitAnd<T> for Output {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn bitand(self, rhs: T) -> Self::Output {
        let graph = self.graph();
        let rhs = self::IntoOutputExt::into_output(rhs, &graph);
        assert_eq!(
            self.signal_type(),
            bool::signal_type(),
            "AND operation requires a boolean signal type"
        );
        assert_eq!(
            self.signal_type(),
            rhs.signal_type(),
            "AND operation requires a boolean signal type"
        );
        let node = graph.node(And::default());
        node.input(0).connect(self);
        node.input(1).connect(rhs);
        node
    }
}

impl<T: IntoOutputExt> std::ops::BitOr<T> for Output {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn bitor(self, rhs: T) -> Self::Output {
        let graph = self.graph();
        let rhs = self::IntoOutputExt::into_output(rhs, &graph);
        assert_eq!(
            self.signal_type(),
            bool::signal_type(),
            "OR operation requires a boolean signal type"
        );
        assert_eq!(
            self.signal_type(),
            rhs.signal_type(),
            "OR operation requires a boolean signal type"
        );
        let node = graph.node(Or::default());
        node.input(0).connect(self);
        node.input(1).connect(rhs);
        node
    }
}

impl std::ops::Not for Output {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn not(self) -> Self::Output {
        let graph = self.graph();
        let node = graph.node(Not::default());
        node.input(0).connect(self);
        node
    }
}

impl<T: IntoOutputExt> std::ops::BitXor<T> for Output {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn bitxor(self, rhs: T) -> Self::Output {
        let graph = self.graph();
        let rhs = self::IntoOutputExt::into_output(rhs, &graph);
        assert_eq!(
            self.signal_type(),
            bool::signal_type(),
            "XOR operation requires a boolean signal type"
        );
        assert_eq!(
            self.signal_type(),
            rhs.signal_type(),
            "XOR operation requires a boolean signal type"
        );
        let node = graph.node(Xor::default());
        node.input(0).connect(self);
        node.input(1).connect(rhs);
        node
    }
}

impl<T: IntoOutputExt> std::ops::Add<T> for Node {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn add(self, rhs: T) -> Self::Output {
        self.output(0).clone() + rhs
    }
}

impl<T: IntoOutputExt> std::ops::Sub<T> for Node {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn sub(self, rhs: T) -> Self::Output {
        self.output(0).clone() - rhs
    }
}

impl<T: IntoOutputExt> std::ops::Mul<T> for Node {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn mul(self, rhs: T) -> Self::Output {
        self.output(0).clone() * rhs
    }
}

impl<T: IntoOutputExt> std::ops::Div<T> for Node {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn div(self, rhs: T) -> Self::Output {
        self.output(0).clone() / rhs
    }
}

impl<T: IntoOutputExt> std::ops::Rem<T> for Node {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn rem(self, rhs: T) -> Self::Output {
        self.output(0).clone() % rhs
    }
}

impl std::ops::Neg for Node {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn neg(self) -> Self::Output {
        self.output(0).neg()
    }
}

impl<T: IntoOutputExt> std::ops::BitAnd<T> for Node {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn bitand(self, rhs: T) -> Self::Output {
        self.output(0).clone() & rhs
    }
}

impl<T: IntoOutputExt> std::ops::BitOr<T> for Node {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn bitor(self, rhs: T) -> Self::Output {
        self.output(0).clone() | rhs
    }
}

impl<T: IntoOutputExt> std::ops::BitXor<T> for Node {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn bitxor(self, rhs: T) -> Self::Output {
        self.output(0).clone() ^ rhs
    }
}

impl std::ops::Not for Node {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn not(self) -> Self::Output {
        !self.output(0).clone()
    }
}

impl<T: IntoOutputExt> std::ops::Add<T> for &Node {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn add(self, rhs: T) -> Self::Output {
        self.clone() + rhs
    }
}

impl<T: IntoOutputExt> std::ops::Sub<T> for &Node {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn sub(self, rhs: T) -> Self::Output {
        self.clone() - rhs
    }
}

impl<T: IntoOutputExt> std::ops::Mul<T> for &Node {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn mul(self, rhs: T) -> Self::Output {
        self.clone() * rhs
    }
}

impl<T: IntoOutputExt> std::ops::Div<T> for &Node {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn div(self, rhs: T) -> Self::Output {
        self.clone() / rhs
    }
}

impl<T: IntoOutputExt> std::ops::Rem<T> for &Node {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn rem(self, rhs: T) -> Self::Output {
        self.clone() % rhs
    }
}

impl std::ops::Neg for &Node {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn neg(self) -> Self::Output {
        self.clone().neg()
    }
}

impl<T: IntoOutputExt> std::ops::BitAnd<T> for &Node {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn bitand(self, rhs: T) -> Self::Output {
        self.clone() & rhs
    }
}

impl<T: IntoOutputExt> std::ops::BitOr<T> for &Node {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn bitor(self, rhs: T) -> Self::Output {
        self.clone() | rhs
    }
}

impl<T: IntoOutputExt> std::ops::BitXor<T> for &Node {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn bitxor(self, rhs: T) -> Self::Output {
        self.clone() ^ rhs
    }
}

impl std::ops::Not for &Node {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn not(self) -> Self::Output {
        !self.clone()
    }
}

/// A trait for coercing a value into an [`Output`].
pub trait IntoOutputExt {
    /// Converts the value into an [`Output`] in the given graph.
    fn into_output(self, graph: &Graph) -> Output;
}

impl IntoOutputExt for Output {
    fn into_output(self, _graph: &Graph) -> Output {
        self
    }
}

impl IntoOutputExt for &Output {
    fn into_output(self, _graph: &Graph) -> Output {
        self.clone()
    }
}

impl IntoOutputExt for Node {
    #[track_caller]
    fn into_output(self, _graph: &Graph) -> Output {
        self.output(0).clone()
    }
}

impl IntoOutputExt for &Node {
    #[track_caller]
    fn into_output(self, _graph: &Graph) -> Output {
        self.output(0).clone()
    }
}

impl<T: Signal + Clone + Default> IntoOutputExt for T {
    #[track_caller]
    fn into_output(self, graph: &Graph) -> Output {
        let node = graph.constant(self);
        node.output(0).clone()
    }
}

impl<T: Signal + Clone> IntoOutputExt for &[T] {
    #[track_caller]
    fn into_output(self, graph: &Graph) -> Output {
        let node = graph.constant(List::from_slice(self));
        node.output(0).clone()
    }
}

impl IntoOutputExt for &str {
    #[track_caller]
    fn into_output(self, graph: &Graph) -> Output {
        let node = graph.constant(Str::from(self));
        node.output(0).clone()
    }
}

impl IntoOutputExt for f64 {
    #[track_caller]
    fn into_output(self, graph: &Graph) -> Output {
        let node = graph.constant(self as f32);
        node.output(0).clone()
    }
}

impl IntoOutputExt for i32 {
    #[track_caller]
    fn into_output(self, graph: &Graph) -> Output {
        let node = graph.constant(self as f32);
        node.output(0).clone()
    }
}

pub trait IntoOutputOpt {
    fn into_output_opt(self, graph: &Graph) -> Option<Output>;
}

impl IntoOutputOpt for Option<Output> {
    fn into_output_opt(self, graph: &Graph) -> Option<Output> {
        if let Some(this) = self.as_ref() {
            assert!(
                this.graph().is_same_graph(graph),
                "Output is from a different graph"
            );
        }
        self
    }
}

impl<T: IntoOutputExt> IntoOutputOpt for T {
    fn into_output_opt(self, graph: &Graph) -> Option<Output> {
        Some(self.into_output(graph))
    }
}

impl IntoOutputOpt for () {
    fn into_output_opt(self, _graph: &Graph) -> Option<Output> {
        None
    }
}

pub trait IntoOutputs {
    fn into_outputs(self, graph: &Graph) -> Vec<Output>;
}

impl<A: IntoOutputExt> IntoOutputs for A {
    fn into_outputs(self, graph: &Graph) -> Vec<Output> {
        vec![self.into_output(graph)]
    }
}

impl<T: IntoOutputExt> IntoOutputs for Vec<T> {
    fn into_outputs(self, graph: &Graph) -> Vec<Output> {
        self.into_iter().map(|o| o.into_output(graph)).collect()
    }
}

macro_rules! impl_into_outputs {
    ($($n:ident),*) => {
        #[allow(non_snake_case)]
        impl<$($n: IntoOutputExt),*> IntoOutputs for ($($n,)*) {
            fn into_outputs(self, graph: &Graph) -> Vec<Output> {
                let ($($n,)*) = self;
                vec![$(
                    $n.into_output(graph),
                )*]
            }
        }
    };
}
impl_into_outputs!(A);
impl_into_outputs!(A, B);
impl_into_outputs!(A, B, C);
impl_into_outputs!(A, B, C, D);
impl_into_outputs!(A, B, C, D, E);
impl_into_outputs!(A, B, C, D, E, F);
impl_into_outputs!(A, B, C, D, E, F, G);
impl_into_outputs!(A, B, C, D, E, F, G, H);

/// A trait for coercing a value into a [`Node`].
pub trait IntoNode {
    /// Converts the value into a [`Node`] in the given graph.
    fn into_node(self, graph: &Graph) -> Node;
}

impl IntoNode for Node {
    #[track_caller]
    fn into_node(self, graph: &Graph) -> Node {
        assert!(
            self.graph().is_same_graph(graph),
            "Nodes belong to different graphs"
        );
        self
    }
}

impl IntoNode for &Node {
    #[track_caller]
    fn into_node(self, graph: &Graph) -> Node {
        assert!(
            self.graph().is_same_graph(graph),
            "Nodes belong to different graphs"
        );
        self.clone()
    }
}

impl IntoNode for NodeIndex {
    fn into_node(self, graph: &Graph) -> Node {
        Node::new(graph.clone(), self)
    }
}
