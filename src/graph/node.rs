//! Contains the [`ProcessorNode`] struct, which represents a node in the audio graph that processes signals.

use std::fmt::Debug;

use crate::{
    prelude::*,
    processor::io::ProcessMode,
    signal::{SignalType, type_erased::ErasedBuffer},
};

use super::{Graph, GraphConstructionResult, NodeIndex};

/// A node in the audio graph that processes signals.
#[derive(Clone)]
pub struct ProcessorNode {
    processor: Box<dyn Processor>,
    input_spec: Vec<SignalSpec>,
    output_spec: Vec<SignalSpec>,
    pub(crate) outputs: Option<Vec<ErasedBuffer>>,
}

impl Debug for ProcessorNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.processor.name())
    }
}

impl ProcessorNode {
    /// Creates a new `ProcessorNode` with the given processor.
    pub fn new(processor: impl Processor) -> Self {
        Self::new_from_boxed(Box::new(processor))
    }

    /// Creates a new `ProcessorNode` with the given boxed processor.
    pub fn new_from_boxed(processor: Box<dyn Processor>) -> Self {
        let input_spec = processor.input_spec();
        let output_spec = processor.output_spec();
        let outputs = processor.create_output_buffers(0);
        Self {
            processor,
            input_spec,
            output_spec,
            outputs: Some(outputs),
        }
    }

    /// Returns the name of the processor.
    #[inline]
    pub fn name(&self) -> &str {
        self.processor.name()
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

    /// Returns a reference to the processor.
    #[inline]
    pub fn processor(&self) -> &dyn Processor {
        &*self.processor
    }

    /// Returns a mutable reference to the processor.
    #[inline]
    pub fn processor_mut(&mut self) -> &mut dyn Processor {
        &mut *self.processor
    }

    /// Allocates memory for the processor.
    #[inline]
    pub fn allocate(&mut self, sample_rate: f32, max_block_size: usize) {
        self.processor.allocate(sample_rate, max_block_size);
        self.outputs = Some(self.processor.create_output_buffers(max_block_size));
    }

    /// Resizes the internal buffers of the processor and updates the sample rate and block size.
    ///
    /// This function is NOT ALLOWED to allocate memory.
    #[inline]
    pub fn resize_buffers(&mut self, sample_rate: f32, block_size: usize) {
        self.processor.resize_buffers(sample_rate, block_size);
    }

    /// Processes the input signals and writes the output signals to the given buffers.
    #[inline]
    pub(crate) fn process(
        &mut self,
        inputs: &[Option<*const ErasedBuffer>],
        env: ProcEnv,
        outputs: &mut [ErasedBuffer],
        mode: ProcessMode,
    ) -> Result<(), ProcessorError> {
        let inputs = ProcessorInputs {
            input_specs: &self.input_spec,
            inputs,
            env,
        };
        let outputs = ProcessorOutputs {
            output_spec: &self.output_spec,
            outputs,
            mode,
        };
        self.processor.process(inputs, outputs)
    }
}

#[inline]
#[track_caller]
fn assert_signals_compatible(a: &SignalType, b: &SignalType, op: impl Into<String>) {
    assert_eq!(
        a,
        b,
        "{}: incompatible signal types: {:?} vs {:?}",
        op.into(),
        a,
        b
    );
}

/// Represents a node in the audio graph. This type is used to build connections between nodes.
#[derive(Clone)]
pub struct Node {
    pub(crate) graph: Graph,
    pub(crate) node_id: NodeIndex,
}

impl Node {
    #[inline]
    pub(crate) fn id(&self) -> NodeIndex {
        self.node_id
    }

    /// Returns the graph builder that this node belongs to.
    #[inline]
    pub fn graph(&self) -> &Graph {
        &self.graph
    }

    /// Returns the name of the processor this node represents.
    #[inline]
    pub fn name(&self) -> String {
        self.graph
            .with_inner(|graph| graph.digraph[self.id()].name().to_string())
    }

    /// Asserts that the node has a single output.
    #[inline]
    #[track_caller]
    pub fn assert_single_output(&self, op: impl Into<String>) {
        assert_eq!(
            self.num_outputs(),
            1,
            "{}: expected single output on node: {}",
            op.into(),
            self.name()
        );
    }

    /// Ensures that the node has a single output, returning an error if it does not.
    #[inline]
    pub fn ensure_single_output(&self, op: impl Into<String>) -> GraphConstructionResult<()> {
        if self.num_outputs() == 1 {
            Ok(())
        } else {
            Err(
                crate::graph::GraphConstructionError::NodeHasMultipleOutputs {
                    op: op.into(),
                    signal_type: self.name(),
                },
            )
        }
    }

    /// Returns the number of inputs of the node.
    #[inline]
    pub fn num_inputs(&self) -> usize {
        self.graph
            .with_inner(|graph| graph.digraph[self.id()].num_inputs())
    }

    /// Returns the number of outputs of the node.
    #[inline]
    pub fn num_outputs(&self) -> usize {
        self.graph
            .with_inner(|graph| graph.digraph[self.id()].num_outputs())
    }

    /// Returns the input of the node at the given index.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    #[inline]
    #[track_caller]
    pub fn input(&self, index: impl IntoInputIdx) -> Input {
        let index = index.into_input_idx(self);
        assert!(
            index < self.num_inputs() as u32,
            "input index {} out of bounds for node {}",
            index,
            self.name()
        );
        Input {
            node: self.clone(),
            input_index: index,
        }
    }

    /// Returns the output of the node at the given index.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    #[inline]
    pub fn output(&self, index: impl IntoOutputIdx) -> Output {
        let index = index.into_output_idx(self);
        assert!(
            index < self.num_outputs() as u32,
            "output index {} out of bounds for node {}",
            index,
            self.name()
        );
        Output {
            node: self.clone(),
            output_index: index,
        }
    }

    /// Returns the signal type of the input at the given index.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    #[inline]
    #[track_caller]
    pub fn input_type(&self, index: impl IntoInputIdx) -> SignalType {
        let index = index.into_input_idx(self);
        assert!(
            index < self.num_inputs() as u32,
            "input index {} out of bounds for node {}",
            index,
            self.name()
        );
        self.graph
            .with_inner(|graph| graph.digraph[self.id()].input_spec()[index as usize].signal_type)
    }

    /// Returns the signal type of the output at the given index.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    #[inline]
    #[track_caller]
    pub fn output_type(&self, index: impl IntoOutputIdx) -> SignalType {
        let index = index.into_output_idx(self);
        assert!(
            index < self.num_outputs() as u32,
            "output index {} out of bounds for node {}",
            index,
            self.name()
        );
        self.graph
            .with_inner(|graph| graph.digraph[self.id()].output_spec()[index as usize].signal_type)
    }

    /// Connects the output of another node to the input of this node.
    ///
    /// # Panics
    ///
    /// - Panics if the input signal type does not match the output signal type.
    /// - Panics if the output index is out of bounds.
    /// - Panics if the input index is out of bounds.
    #[inline]
    #[track_caller]
    pub fn connect_input(
        &self,
        source: impl IntoNode,
        source_output: impl IntoOutputIdx,
        target_input: impl IntoInputIdx,
    ) -> Node {
        let output = source.into_node(&self.graph);
        let source_output = source_output.into_output_idx(&output);
        let target_input = target_input.into_input_idx(self);

        assert_signals_compatible(
            &output.output_type(source_output),
            &self.input_type(target_input),
            "connect_input",
        );
        assert!(
            target_input < self.num_inputs() as u32,
            "input index {} out of bounds for node {}",
            target_input,
            self.name()
        );
        assert!(
            source_output < output.num_outputs() as u32,
            "output index {} out of bounds for node {}",
            source_output,
            output.name()
        );

        self.graph
            .connect(output.id(), source_output, self.id(), target_input);
        self.clone()
    }

    /// Connects the output of this node to the input of another node.
    ///
    /// # Panics
    ///
    /// - Panics if the output signal type does not match the input signal type.
    /// - Panics if the output index is out of bounds.
    /// - Panics if the input index is out of bounds.
    #[inline]
    #[track_caller]
    pub fn connect_output(
        &self,
        output: impl IntoOutputIdx,
        target: impl IntoNode,
        target_input: impl IntoInputIdx,
    ) -> Node {
        let target = target.into_node(&self.graph);
        let output_index = output.into_output_idx(self);
        let target_input = target_input.into_input_idx(&target);

        assert_signals_compatible(
            &self.output_type(output_index),
            &target.input_type(target_input),
            "connect_output",
        );
        assert!(
            output_index < self.num_outputs() as u32,
            "output index {} out of bounds for node {}",
            output_index,
            self.name()
        );
        assert!(
            target_input < target.num_inputs() as u32,
            "input index {} out of bounds for node {}",
            target_input,
            target.name()
        );

        self.graph
            .connect(self.id(), output_index, target.id(), target_input);
        self.clone()
    }
}

/// Represents an input of a [`Node`].
#[derive(Clone)]
pub struct Input {
    pub(crate) node: Node,
    pub(crate) input_index: u32,
}

impl Input {
    /// Returns the signal type of the input.
    #[inline]
    pub fn signal_type(&self) -> SignalType {
        self.node.input_type(self.input_index)
    }

    /// Returns the [`Node`] that this input is connected to.
    #[inline]
    pub fn node(&self) -> Node {
        self.node.clone()
    }

    /// Returns the index of the input.
    #[inline]
    pub fn index(&self) -> u32 {
        self.input_index
    }

    /// Connects the input to the output of another node.
    ///
    /// # Panics
    ///
    /// Panics if the output and input signals do not have the same type.
    #[inline]
    #[track_caller]
    pub fn connect(&self, output: impl IntoOutput) -> Node {
        let output = output.into_output(self.node.graph());
        assert_signals_compatible(&output.signal_type(), &self.signal_type(), "connect");
        self.node
            .connect_input(&output.node, output.output_index, self.input_index);
        self.node.clone()
    }
}

macro_rules! choose_node_generics {
    ($graph:expr, $signal_type:expr => $node_type:ident => $($options:ty)*) => {
        match $signal_type {
            $(
                t if t == <$options>::signal_type() => $graph.add($node_type::<$options>::default()),
            )*
            _ => panic!("Unsupported signal type: {:?}", $signal_type),
        }
    };
}

macro_rules! generic_binary_op_impl {
    ($self:ident, $b:ident, $op:ident => $($options:ty)*) => {{
        let this_node = $self.node();
        let graph = this_node.graph();
        let b = $b.into_output(graph);
        assert_eq!(
            $self.signal_type(),
            b.signal_type(),
            "Signal types must match for this operation",
        );
        let node = choose_node_generics!(graph, $self.signal_type() => $op => $($options)*);
        node.input(0).connect($self);
        node.input(1).connect(b);
        node
    }};
}

/// Represents an output of a [`Node`].
#[derive(Clone)]
pub struct Output {
    pub(crate) node: Node,
    pub(crate) output_index: u32,
}

impl Output {
    /// Returns the [`Node`] that this output is connected to.
    #[inline]
    pub fn node(&self) -> Node {
        self.node.clone()
    }

    /// Returns the index of the output.
    #[inline]
    pub fn index(&self) -> u32 {
        self.output_index
    }

    /// Returns the signal type of the output.
    #[inline]
    pub fn signal_type(&self) -> SignalType {
        self.node.output_type(self.output_index)
    }

    /// Connects the output to the input of another node.
    ///
    /// # Panics
    ///
    /// Panics if the output and input signals do not have the same type.
    #[inline]
    #[track_caller]
    pub fn connect(&self, input: &Input) -> Node {
        assert_signals_compatible(&self.signal_type(), &input.signal_type(), "connect");
        self.node
            .connect_output(self.output_index, &input.node, input.input_index);
        self.node.clone()
    }

    #[inline]
    pub fn add(&self, b: impl IntoOutput) -> Node {
        generic_binary_op_impl!(self, b, Add => f32 i64)
    }

    #[inline]
    pub fn sub(&self, b: impl IntoOutput) -> Node {
        generic_binary_op_impl!(self, b, Sub => f32 i64)
    }

    #[inline]
    pub fn mul(&self, b: impl IntoOutput) -> Node {
        generic_binary_op_impl!(self, b, Mul => f32 i64)
    }

    #[inline]
    pub fn div(&self, b: impl IntoOutput) -> Node {
        generic_binary_op_impl!(self, b, Div => f32 i64)
    }

    #[inline]
    pub fn rem(&self, b: impl IntoOutput) -> Node {
        generic_binary_op_impl!(self, b, Rem => f32 i64)
    }

    #[inline]
    pub fn neg(&self) -> Node {
        let this_node = self.node();
        let graph = this_node.graph();
        let node = choose_node_generics!(graph, self.signal_type() => Neg => f32 i64);
        node.input(0).connect(self);
        node
    }
}

impl<T: IntoOutput> std::ops::Add<T> for Output {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn add(self, rhs: T) -> Self::Output {
        generic_binary_op_impl!(self, rhs, Add => f32 i64)
    }
}

impl<T: IntoOutput> std::ops::Sub<T> for Output {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn sub(self, rhs: T) -> Self::Output {
        generic_binary_op_impl!(self, rhs, Sub => f32 i64)
    }
}

impl<T: IntoOutput> std::ops::Mul<T> for Output {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn mul(self, rhs: T) -> Self::Output {
        generic_binary_op_impl!(self, rhs, Mul => f32 i64)
    }
}

impl<T: IntoOutput> std::ops::Div<T> for Output {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn div(self, rhs: T) -> Self::Output {
        generic_binary_op_impl!(self, rhs, Div => f32 i64)
    }
}

impl<T: IntoOutput> std::ops::Rem<T> for Output {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn rem(self, rhs: T) -> Self::Output {
        generic_binary_op_impl!(self, rhs, Rem => f32 i64)
    }
}

impl std::ops::Neg for Output {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn neg(self) -> Self::Output {
        let this_node = self.node();
        let graph = this_node.graph();
        let node = choose_node_generics!(graph, self.signal_type() => Neg => f32 i64);
        node.input(0).connect(self);
        node
    }
}

impl<T: IntoOutput> std::ops::Add<T> for Node {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn add(self, rhs: T) -> Self::Output {
        self.assert_single_output("add");
        self.output(0) + rhs
    }
}

impl<T: IntoOutput> std::ops::Sub<T> for Node {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn sub(self, rhs: T) -> Self::Output {
        self.assert_single_output("sub");
        self.output(0) - rhs
    }
}

impl<T: IntoOutput> std::ops::Mul<T> for Node {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn mul(self, rhs: T) -> Self::Output {
        self.assert_single_output("mul");
        self.output(0) * rhs
    }
}

impl<T: IntoOutput> std::ops::Div<T> for Node {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn div(self, rhs: T) -> Self::Output {
        self.assert_single_output("div");
        self.output(0) / rhs
    }
}

impl<T: IntoOutput> std::ops::Rem<T> for Node {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn rem(self, rhs: T) -> Self::Output {
        self.assert_single_output("rem");
        self.output(0) % rhs
    }
}

impl std::ops::Neg for Node {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn neg(self) -> Self::Output {
        self.assert_single_output("neg");
        self.output(0).neg()
    }
}

impl<T: IntoOutput> std::ops::Add<T> for &Node {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn add(self, rhs: T) -> Self::Output {
        self.clone() + rhs
    }
}

impl<T: IntoOutput> std::ops::Sub<T> for &Node {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn sub(self, rhs: T) -> Self::Output {
        self.clone() - rhs
    }
}

impl<T: IntoOutput> std::ops::Mul<T> for &Node {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn mul(self, rhs: T) -> Self::Output {
        self.clone() * rhs
    }
}

impl<T: IntoOutput> std::ops::Div<T> for &Node {
    type Output = Node;

    #[inline]
    #[track_caller]
    fn div(self, rhs: T) -> Self::Output {
        self.clone() / rhs
    }
}

impl<T: IntoOutput> std::ops::Rem<T> for &Node {
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

mod sealed {
    use crate::signal::Signal;

    pub trait Sealed {}
    impl Sealed for crate::graph::NodeIndex {}
    impl Sealed for super::Node {}
    impl Sealed for &super::Node {}
    impl Sealed for super::Output {}
    impl Sealed for &super::Output {}
    impl<T: Signal> Sealed for T {}
    impl Sealed for i32 {}
    impl Sealed for u32 {}
    impl Sealed for usize {}
    impl Sealed for &str {}
}

/// A trait for coercing a value into an [`Output`].
pub trait IntoOutput: sealed::Sealed {
    /// Converts the value into an [`Output`] in the given graph.
    fn into_output(self, graph: &Graph) -> Output;
}

impl IntoOutput for Output {
    fn into_output(self, _graph: &Graph) -> Output {
        self
    }
}

impl IntoOutput for &Output {
    fn into_output(self, _graph: &Graph) -> Output {
        self.clone()
    }
}

impl<T: IntoNode> IntoOutput for T {
    #[track_caller]
    fn into_output(self, graph: &Graph) -> Output {
        let node = self.into_node(graph);
        node.assert_single_output("into_output");
        node.output(0)
    }
}

impl IntoOutput for f32 {
    fn into_output(self, graph: &Graph) -> Output {
        let node = graph.constant(self);
        node.output(0)
    }
}

impl IntoOutput for i32 {
    fn into_output(self, graph: &Graph) -> Output {
        let node = graph.constant(self as i64);
        node.output(0)
    }
}

impl IntoOutput for i64 {
    fn into_output(self, graph: &Graph) -> Output {
        let node = graph.constant(self);
        node.output(0)
    }
}

impl IntoOutput for usize {
    fn into_output(self, graph: &Graph) -> Output {
        let node = graph.constant(self as i64);
        node.output(0)
    }
}

/// A trait for coercing a value into a [`Node`].
pub trait IntoNode: sealed::Sealed {
    /// Converts the value into a [`Node`] in the given graph.
    fn into_node(self, graph: &Graph) -> Node;
}

impl IntoNode for Node {
    fn into_node(self, graph: &Graph) -> Node {
        Node {
            graph: graph.clone(),
            node_id: self.node_id,
        }
    }
}

impl IntoNode for &Node {
    fn into_node(self, graph: &Graph) -> Node {
        Node {
            graph: graph.clone(),
            node_id: self.node_id,
        }
    }
}

impl IntoNode for NodeIndex {
    fn into_node(self, graph: &Graph) -> Node {
        Node {
            graph: graph.clone(),
            node_id: self,
        }
    }
}

/// A trait for coercing a value into an output index of a node.
pub trait IntoOutputIdx: sealed::Sealed {
    /// Converts the value into an output index of the given node.
    fn into_output_idx(self, node: &Node) -> u32;
}

/// A trait for coercing a value into an input index of a node.
pub trait IntoInputIdx: sealed::Sealed {
    /// Converts the value into an input index of the given node.
    fn into_input_idx(self, node: &Node) -> u32;
}

impl IntoOutputIdx for u32 {
    #[inline]
    fn into_output_idx(self, node: &Node) -> u32 {
        assert!(
            self < node.num_outputs() as u32,
            "output index out of bounds"
        );
        self
    }
}

impl IntoInputIdx for u32 {
    #[inline]
    fn into_input_idx(self, node: &Node) -> u32 {
        assert!(self < node.num_inputs() as u32, "input index out of bounds");
        self
    }
}

impl IntoInputIdx for &str {
    #[track_caller]
    #[inline]
    fn into_input_idx(self, node: &Node) -> u32 {
        let Some(idx) = node.graph().with_inner(|graph| {
            graph.digraph[node.id()]
                .input_spec()
                .iter()
                .position(|s| s.name == self)
        }) else {
            panic!("no input with name {self}")
        };
        idx as u32
    }
}

impl IntoOutputIdx for &str {
    #[track_caller]
    #[inline]
    fn into_output_idx(self, node: &Node) -> u32 {
        let Some(idx) = node.graph().with_inner(|graph| {
            graph.digraph[node.id()]
                .output_spec()
                .iter()
                .position(|s| s.name == self)
        }) else {
            panic!("no output with name {self}")
        };
        idx as u32
    }
}
