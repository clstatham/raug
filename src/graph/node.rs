//! Contains the [`ProcessorNode`] struct, which represents a node in the audio graph that processes signals.

use std::fmt::Debug;

use crate::{
    prelude::*,
    processor::io::ProcessMode,
    signal::{Signal, SignalType, buffer::SignalBuffer},
};

use super::{Graph, GraphConstructionResult, NodeIndex};

/// A node in the audio graph that processes signals.
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProcessorNode {
    processor: Box<dyn Processor>,
    input_spec: Vec<SignalSpec>,
    output_spec: Vec<SignalSpec>,
    pub(crate) outputs: Option<Vec<SignalBuffer>>,
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
        let mut outputs = Vec::with_capacity(output_spec.len());
        for spec in output_spec.iter() {
            outputs.push(SignalBuffer::new_of_type(spec.signal_type, 0));
        }
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
        for (spec, output) in self
            .output_spec
            .iter()
            .zip(self.outputs.as_mut().unwrap().iter_mut())
        {
            output.resize_with_hint(max_block_size, spec.signal_type);
        }
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
        inputs: &[Option<*const SignalBuffer>],
        env: ProcEnv,
        outputs: &mut [SignalBuffer],
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

    /// Connects a [`Smooth`] processor to the output of this node.
    ///
    /// The `factor` parameter controls the smoothing factor, where a value of 0.0 means maximum smoothing and 1.0 means no smoothing.
    ///
    /// # Panics
    ///
    /// Panics if the node has multiple outputs.
    #[inline]
    #[track_caller]
    pub fn smooth(&self, factor: f32) -> Node {
        self.assert_single_output("smooth");
        self.output(0).smooth(factor)
    }

    /// Connects a [`MidiToFreq`] processor to the output of this node.
    ///
    /// # Panics
    ///
    /// Panics if the node has multiple outputs.
    #[inline]
    #[track_caller]
    pub fn midi2freq(&self) -> Node {
        self.assert_single_output("midi2freq");
        self.output(0).midi2freq()
    }

    /// Connects a [`FreqToMidi`] processor to the output of this node.
    ///
    /// # Panics
    ///
    /// Panics if the node has multiple outputs.
    #[inline]
    #[track_caller]
    pub fn freq2midi(&self) -> Node {
        self.assert_single_output("freq2midi");
        self.output(0).freq2midi()
    }

    /// Connects a [`Register`] processor to the output of this node.
    ///
    /// The register processor stores the last value of the input signal and continuously outputs it.
    /// Useful for "remembering" a value across multiple frames.
    ///
    /// # Panics
    ///
    /// Panics if the node has multiple outputs.
    #[inline]
    #[track_caller]
    pub fn make_register(&self) -> Node {
        self.assert_single_output("make_register");
        self.output(0).make_register()
    }

    /// Connects a [`Cond`] processor to the output of this node.
    ///
    /// The `then` and `else_` parameters are the signals to output when the condition is true or false, respectively.
    ///
    /// # Panics
    ///
    /// - Panics if the node has multiple outputs.
    /// - Panics if the output signals do not have the same type.
    /// - Panics if the node's output is not a boolean signal.
    #[inline]
    #[track_caller]
    pub fn cond(&self, then: impl IntoNode, else_: impl IntoNode) -> Node {
        self.assert_single_output("cond");
        self.output(0).cond(then, else_)
    }

    /// Connects a [`Cast`] processor to the output of this node.
    ///
    /// The `signal_type` parameter specifies the type to cast the signal to.
    ///
    /// # Panics
    ///
    /// - Panics if the node has multiple outputs.
    /// - Panics if the output signal cannot be cast to the specified type.
    #[inline]
    #[track_caller]
    pub fn cast(&self, signal_type: SignalType) -> Node {
        self.assert_single_output("cast");
        self.output(0).cast(signal_type)
    }

    /// Connects a [`Dedup`] processor to the output of this node.
    ///
    /// # Panics
    ///
    /// - Panics if the node has multiple outputs.
    #[inline]
    #[track_caller]
    pub fn dedup(&self) -> Node {
        self.assert_single_output("dedup");
        self.output(0).dedup()
    }

    /// Connects a [`Print`] processor to the output of this node.
    ///
    /// # Panics
    ///
    /// - Panics if the node has multiple outputs.
    /// - Panics if the output signal is not a float.
    #[inline]
    #[track_caller]
    pub fn print(&self) -> Node {
        self.assert_single_output("print");
        self.output(0).print()
    }

    /// Connects a [`CheckFinite`] processor to the output of this node.
    ///
    /// # Panics
    ///
    /// - Panics if the node has multiple outputs.
    /// - Panics if the output signal is not a float.
    ///
    /// Also note that, during the execution of the graph, this processor will panic if the output signal is `inf` or `NaN`.
    #[inline]
    #[track_caller]
    pub fn check_finite(&self) -> Node {
        self.assert_single_output("check_finite");
        self.output(0).check_finite()
    }

    /// Connects a [`FiniteOrZero`] processor to the output of this node.
    ///
    /// # Panics
    ///
    /// - Panics if the node has multiple outputs.
    /// - Panics if the output signal is not a float.
    #[inline]
    #[track_caller]
    pub fn finite_or_zero(&self) -> Node {
        self.assert_single_output("finite_or_zero");
        self.output(0).finite_or_zero()
    }

    /// Connects a [`IsSome`] processor to the output of this node.
    ///
    /// # Panics
    ///
    /// - Panics if the node has multiple outputs.
    #[inline]
    #[track_caller]
    pub fn is_some(&self) -> Node {
        self.assert_single_output("is_some");
        self.output(0).is_some()
    }

    /// Connects a [`IsNone`] processor to the output of this node.
    ///
    /// # Panics
    ///
    /// - Panics if the node has multiple outputs.
    #[inline]
    #[track_caller]
    pub fn is_none(&self) -> Node {
        self.assert_single_output("is_none");
        self.output(0).is_none()
    }

    /// Connects a [`OrElse`] processor to the output of this node.
    ///
    /// # Panics
    ///
    /// - Panics if the node has multiple outputs.
    #[inline]
    #[track_caller]
    pub fn or_else(&self, default: impl Signal) -> Node {
        self.assert_single_output("or_else");
        self.output(0).or_else(default)
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

    /// Creates a [`Param`] processor and connects it to the input.
    ///
    /// This can be used to create a parameter that can be controlled externally.
    ///
    /// # Panics
    ///
    /// Panics if the input signal type does not match the initial value signal type (type parameter `S`).
    #[inline]
    pub fn param<S: Signal + Clone>(
        &self,
        name: impl Into<String>,
        initial_value: impl Into<Option<S>>,
    ) -> Param {
        let name = name.into();
        let param = Param::new::<S>(&name, initial_value);
        let proc = self.node.graph().add_param(param.clone());
        proc.output(0).connect(self);
        param
    }
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

    /// Creates a [`Cast`] processor and connects it to the output.
    ///
    /// The `signal_type` parameter specifies the type to cast the signal to.
    ///
    /// # Panics
    ///
    /// Panics if the output signal cannot be cast to the specified type.
    #[inline]
    pub fn cast(&self, signal_type: SignalType) -> Node {
        let current_type = self.signal_type();
        if current_type == signal_type {
            return self.node.clone();
        }
        let cast = self.node.graph().add(Cast::new(current_type, signal_type));

        cast.input(0).connect(self);
        cast
    }

    /// Creates a [`Passthrough`] processor and connects it to the output.
    ///
    /// This can be useful in situations where a [`Node`] is required instead of an [`Output`].
    #[inline]
    pub fn make_node(&self) -> Node {
        let signal_type = self.signal_type();
        let node = self.node.graph().add(Passthrough::new(signal_type));
        node.input(0).connect(self);
        node
    }

    /// Creates a [`Register`] processor and connects it to the output.
    ///
    /// The register processor stores the last value of the input signal and continuously outputs it.
    /// Useful for "remembering" a value across multiple frames.
    #[inline]
    pub fn make_register(&self) -> Node {
        let signal_type = self.signal_type();
        let node = self.node.graph().add(Register::new(signal_type));
        node.input(0).connect(self);
        node
    }

    /// Creates a [`Smooth`] processor and connects it to the output.
    ///
    /// The `factor` parameter controls the smoothing factor, where a value of 0.0 means maximum smoothing and 1.0 means no smoothing.
    #[inline]
    pub fn smooth(&self, factor: impl IntoOutput) -> Node {
        let factor = factor.into_output(self.node.graph());
        let proc = self.node.graph().add(Smooth::default());
        proc.input("factor").connect(factor);
        proc.input(0).connect(self);
        proc
    }

    /// Creates a [`MidiToFreq`] processor and connects it to the output.
    #[inline]
    pub fn midi2freq(&self) -> Node {
        let proc = self.node.graph().add(MidiToFreq);
        proc.input(0).connect(self);
        proc
    }

    /// Creates a [`FreqToMidi`] processor and connects it to the output.
    #[inline]
    pub fn freq2midi(&self) -> Node {
        let proc = self.node.graph().add(FreqToMidi);
        proc.input(0).connect(self);
        proc
    }

    /// Creates a [`Cond`] processor and connects it to the output.
    ///
    /// The `then` and `else_` parameters are the signals to output when the condition is true or false, respectively.
    ///
    /// # Panics
    ///
    /// - Panics if the output signals do not have the same type.
    /// - Panics if the node's output is not a boolean signal.
    #[inline]
    #[track_caller]
    pub fn cond(&self, then: impl IntoOutput, else_: impl IntoOutput) -> Node {
        let then = then.into_output(self.node.graph());
        let else_ = else_.into_output(self.node.graph());
        let signal_type = then.signal_type();
        assert_signals_compatible(&signal_type, &else_.signal_type(), "cond");
        assert!(
            self.signal_type() == SignalType::Bool,
            "condition signal must be a boolean"
        );
        let cond = self.node.graph().add(Cond::new(signal_type));
        cond.input("cond").connect(self);
        cond.input("then").connect(&then);
        cond.input("else").connect(&else_);
        cond
    }

    /// Creates a [`Dedup`] processor and connects it to the output.
    #[inline]
    pub fn dedup(&self) -> Node {
        let signal_type = self.signal_type();
        let proc = self.node.graph().add(Dedup::new(signal_type));
        proc.input(0).connect(self);
        proc
    }

    /// Creates a [`Print`] processor and connects it to the output.
    ///
    /// # Panics
    ///
    /// Panics if the output signal is not a float.
    #[inline]
    #[track_caller]
    pub fn print(&self) -> Node {
        assert!(
            matches!(self.signal_type(), SignalType::Float),
            "output signal must be a float"
        );
        let proc = self.node.graph().add(Print::new(SignalType::Float));
        let changed = self.node().graph().add(Changed::new(0.0, true));
        changed.input(0).connect(self);
        proc.input("trig").connect(changed);
        proc.input("message").connect(self);
        proc
    }

    /// Creates a [`CheckFinite`] processor and connects it to the output.
    ///
    /// # Panics
    ///
    /// Panics if the output signal is not a float.
    ///
    /// Also note that, during the execution of the graph, this processor will panic if the output signal is `inf` or `NaN`.
    #[inline]
    #[track_caller]
    pub fn check_finite(&self) -> Node {
        assert!(
            matches!(self.signal_type(), SignalType::Float),
            "output signal must be a float"
        );
        let proc = self.node.graph().add(CheckFinite::default());
        proc.input(0).connect(self);
        proc
    }

    /// Creates a [`FiniteOrZero`] processor and connects it to the output.
    ///
    /// # Panics
    ///
    /// Panics if the output signal is not a float.
    #[inline]
    #[track_caller]
    pub fn finite_or_zero(&self) -> Node {
        assert!(
            matches!(self.signal_type(), SignalType::Float),
            "output signal must be a float"
        );
        let proc = self.node.graph().add(FiniteOrZero::default());
        proc.input(0).connect(self);
        proc
    }

    /// Creates a [`IsSome`] processor and connects it to the output.
    #[inline]
    pub fn is_some(&self) -> Node {
        let proc = self.node.graph().add(IsSome::new(self.signal_type()));
        proc.input(0).connect(self);
        proc
    }

    /// Creates a [`IsNone`] processor and connects it to the output.
    #[inline]
    pub fn is_none(&self) -> Node {
        let proc = self.node.graph().add(IsNone::new(self.signal_type()));
        proc.input(0).connect(self);
        proc
    }

    /// Creates a [`OrElse`] processor and connects it to the output.
    #[inline]
    pub fn or_else(&self, default: impl Signal) -> Node {
        let proc = self.node.graph().add(OrElse::new(default));
        proc.input(0).connect(self);
        proc
    }
}

mod sealed {
    pub trait Sealed {}
    impl Sealed for crate::graph::NodeIndex {}
    impl Sealed for super::Node {}
    impl Sealed for &super::Node {}
    impl Sealed for super::Output {}
    impl Sealed for &super::Output {}
    impl Sealed for super::AnySignal {}
    impl Sealed for crate::builtins::util::Param {}
    impl Sealed for f32 {}
    impl Sealed for bool {}
    impl Sealed for i32 {}
    impl Sealed for i64 {}
    impl Sealed for u32 {}
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

/// A trait for coercing a value into a [`Node`].
pub trait IntoNode: sealed::Sealed {
    /// Converts the value into a [`Node`] in the given graph.
    fn into_node(self, graph: &Graph) -> Node;
}

impl IntoNode for AnySignal {
    fn into_node(self, graph: &Graph) -> Node {
        graph.add(Constant::new_any(self))
    }
}

impl IntoNode for bool {
    fn into_node(self, graph: &Graph) -> Node {
        graph.constant(self)
    }
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

impl IntoNode for Param {
    fn into_node(self, graph: &Graph) -> Node {
        graph.add(self)
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

impl IntoNode for f32 {
    fn into_node(self, graph: &Graph) -> Node {
        graph.constant(self)
    }
}

impl IntoNode for i64 {
    fn into_node(self, graph: &Graph) -> Node {
        graph.constant(self)
    }
}

impl IntoNode for i32 {
    fn into_node(self, graph: &Graph) -> Node {
        graph.constant(self as i64)
    }
}

impl IntoNode for u32 {
    fn into_node(self, graph: &Graph) -> Node {
        graph.constant(self as i64)
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

macro_rules! impl_binary_node_ops {
    ($name:ident, $proc:ident, ($($signal_type:ident => $data:ty),*), $doc:literal) => {
        impl Output {
            #[allow(clippy::should_implement_trait)]
            #[doc = $doc]
            pub fn $name(&self, other: impl IntoOutput) -> Node {
                let other = other.into_output(self.node().graph());

                assert_signals_compatible(
                    &self.signal_type(),
                    &other.signal_type(),
                    stringify!($name),
                );

                let signal_type = self.signal_type();
                let node = self.node().graph().add(<math::$proc>::new(signal_type));

                node.input(0).connect(self);
                node.input(1).connect(&other);

                node
            }
        }

        impl Node {
            #[allow(clippy::should_implement_trait)]
            #[doc = $doc]
            pub fn $name(&self, other: impl IntoOutput) -> Node {
                self.assert_single_output(stringify!($name));
                self.output(0).$name(other)
            }
        }
    };
    ($name:ident, $std_op:ident, $proc:ident, ($($signal_type:ident => $data:ty),*), $doc:literal) => {
        impl Output {
            #[allow(clippy::should_implement_trait)]
            #[doc = $doc]
            pub fn $name(&self, other: impl IntoOutput) -> Node {
                let other = other.into_output(self.node().graph());

                assert_signals_compatible(
                    &self.signal_type(),
                    &other.signal_type(),
                    stringify!($name),
                );

                let signal_type = self.signal_type();
                let node = self.node().graph().add(<math::$proc>::new(signal_type));

                node.input(0).connect(self);
                node.input(1).connect(&other);

                node
            }
        }

        impl Node {
            #[allow(clippy::should_implement_trait)]
            #[doc = $doc]
            pub fn $name(&self, other: impl IntoOutput) -> Node {
                self.assert_single_output(stringify!($name));
                self.output(0).$name(other)
            }
        }

        impl<T> std::ops::$std_op<T> for Output
        where
            T: IntoOutput,
        {
            type Output = Node;

            fn $name(self, other: T) -> Node {
                Output::$name(&self, other)
            }
        }

        impl<T> std::ops::$std_op<T> for &Output
        where
            T: IntoOutput,
        {
            type Output = Node;

            fn $name(self, other: T) -> Node {
                Output::$name(self, other)
            }
        }

        impl<T> std::ops::$std_op<T> for Node
        where
            T: IntoNode,
        {
            type Output = Node;

            fn $name(self, other: T) -> Node {
                Node::$name(&self, other)
            }
        }

        impl<T> std::ops::$std_op<T> for &Node
        where
            T: IntoNode,
        {
            type Output = Node;

            fn $name(self, other: T) -> Node {
                Node::$name(self, other)
            }
        }
    };
}

impl_binary_node_ops!(add, Add, Add, (f32 => f32, Int => i64), "Adds two signals together.");
impl_binary_node_ops!(sub, Sub, Sub, (f32 => f32, Int => i64), "Subtracts one signal from another.");
impl_binary_node_ops!(mul, Mul, Mul, (f32 => f32, Int => i64), "Multiplies two signals together.");
impl_binary_node_ops!(div, Div, Div, (f32 => f32, Int => i64), "Divides one signal by another.");
impl_binary_node_ops!(
    rem,
    Rem,
    Rem,
    (f32 => f32, Int => i64),
    "Calculates the remainder of one signal divided by another."
);
impl_binary_node_ops!(powf, Powf, (f32 => f32), "Raises one signal to the power of another.");
impl_binary_node_ops!(
    atan2,
    Atan2,
    (f32 => f32),
    "Calculates the arctangent of the ratio of two signals."
);
impl_binary_node_ops!(hypot, Hypot, (f32 => f32), "Calculates the hypotenuse of two signals.");
impl_binary_node_ops!(max, Max, (f32 => f32, Int => i64), "Outputs the maximum of two signals.");
impl_binary_node_ops!(min, Min, (f32 => f32, Int => i64), "Outputs the minimum of two signals.");

macro_rules! impl_comparison_node_ops {
    ($name:ident, $proc:ident, $doc:expr) => {
        impl Output {
            #[allow(clippy::should_implement_trait)]
            #[doc = $doc]
            pub fn $name(&self, other: impl IntoOutput) -> Node {
                let other = other.into_output(self.node().graph());

                assert_signals_compatible(
                    &self.signal_type(),
                    &other.signal_type(),
                    stringify!($name),
                );

                let signal_type = self.signal_type();
                let node = self.node().graph().add(<control::$proc>::new(signal_type));

                node.input(0).connect(self);
                node.input(1).connect(&other);

                node
            }
        }

        impl Node {
            #[allow(clippy::should_implement_trait)]
            #[doc = $doc]
            pub fn $name(&self, other: impl IntoOutput) -> Node {
                self.assert_single_output(stringify!($name));
                self.output(0).$name(other)
            }
        }
    };
}

impl_comparison_node_ops!(eq, Equal, "Outputs true if the two signals are equal.");
impl_comparison_node_ops!(
    ne,
    NotEqual,
    "Outputs true if the two signals are not equal."
);
impl_comparison_node_ops!(
    lt,
    Less,
    "Outputs true if the first signal is less than the second signal."
);
impl_comparison_node_ops!(
    le,
    LessOrEqual,
    "Outputs true if the first signal is less than or equal to the second signal."
);
impl_comparison_node_ops!(
    gt,
    Greater,
    "Outputs true if the first signal is greater than the second signal."
);
impl_comparison_node_ops!(
    ge,
    GreaterOrEqual,
    "Outputs true if the first signal is greater than or equal to the second signal."
);

macro_rules! impl_unary_node_ops {
    ($name:ident, $proc:ident, ($($signal_type:ident => $data:ty),*), $doc:literal) => {
        impl Output {
            #[allow(clippy::should_implement_trait)]
            #[doc = $doc]
            pub fn $name(&self) -> Node {
                let signal_type = self.signal_type();
                let node = self.node().graph().add(<math::$proc>::new(signal_type));

                node.input(0).connect(self);

                node
            }
        }

        impl Node {
            #[allow(clippy::should_implement_trait)]
            #[doc = $doc]
            pub fn $name(&self) -> Node {
                self.assert_single_output(stringify!($name));
                self.output(0).$name()
            }
        }
    };
}

impl_unary_node_ops!(neg, Neg, (f32 => f32, Int => i64), "Negates the input signal.");

impl std::ops::Neg for &Node {
    type Output = Node;

    fn neg(self) -> Node {
        Node::neg(self)
    }
}

impl_unary_node_ops!(
    abs,
    Abs,
    (f32 => f32, Int => i64),
    "Outputs the absolute value of the input signal."
);
impl_unary_node_ops!(
    sqrt,
    Sqrt,
    (f32 => f32),
    "Outputs the square root of the input signal."
);
impl_unary_node_ops!(
    cbrt,
    Cbrt,
    (f32 => f32),
    "Outputs the cube root of the input signal."
);
impl_unary_node_ops!(
    ceil,
    Ceil,
    (f32 => f32),
    "Rounds the input signal up to the nearest integer."
);
impl_unary_node_ops!(
    floor,
    Floor,
    (f32 => f32),
    "Rounds the input signal down to the nearest integer."
);
impl_unary_node_ops!(
    round,
    Round,
    (f32 => f32),
    "Rounds the input signal to the nearest integer."
);
impl_unary_node_ops!(sin, Sin, (f32 => f32), "Outputs the sine of the input signal.");
impl_unary_node_ops!(cos, Cos, (f32 => f32), "Outputs the cosine of the input signal.");
impl_unary_node_ops!(tan, Tan, (f32 => f32), "Outputs the tangent of the input signal.");
impl_unary_node_ops!(
    tanh,
    Tanh,
    (f32 => f32),
    "Outputs the hyperbolic tangent of the input signal."
);

impl_unary_node_ops!(
    recip,
    Recip,
    (f32 => f32),
    "Outputs the reciprocal of the input signal."
);
impl_unary_node_ops!(
    signum,
    Signum,
    (f32 => f32, Int => i64),
    "Outputs the sign of the input signal."
);
impl_unary_node_ops!(
    fract,
    Fract,
    (f32 => f32),
    "Outputs the fractional part of the input signal."
);
impl_unary_node_ops!(
    ln,
    Ln,
    (f32 => f32),
    "Outputs the natural logarithm of the input signal."
);
impl_unary_node_ops!(
    log2,
    Log2,
    (f32 => f32),
    "Outputs the base-2 logarithm of the input signal."
);
impl_unary_node_ops!(
    log10,
    Log10,
    (f32 => f32),
    "Outputs the base-10 logarithm of the input signal."
);
impl_unary_node_ops!(
    exp,
    Exp,
    (f32 => f32),
    "Outputs the natural exponential of the input signal."
);
