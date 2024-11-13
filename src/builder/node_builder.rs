//! Contains the [`Node`] type and related types and traits.

use petgraph::prelude::*;

use crate::{
    graph::GraphConstructionResult,
    prelude::*,
    signal::{Signal, SignalType},
};

use super::graph_builder::GraphBuilder;

/// Represents a node in the audio graph. This type is used to build connections between nodes.
#[derive(Clone)]
pub struct Node {
    pub(crate) graph: GraphBuilder,
    pub(crate) node_id: NodeIndex,
}

impl Node {
    #[inline]
    pub(crate) fn id(&self) -> NodeIndex {
        self.node_id
    }

    /// Returns the graph builder that this node belongs to.
    #[inline]
    pub fn graph(&self) -> &GraphBuilder {
        &self.graph
    }

    /// Returns the name of the processor this node represents.
    #[inline]
    pub fn name(&self) -> String {
        self.graph
            .with_graph(|graph| graph.digraph()[self.id()].name().to_string())
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
                    type_: self.name(),
                },
            )
        }
    }

    /// Returns the number of inputs of the node.
    #[inline]
    pub fn num_inputs(&self) -> usize {
        self.graph
            .with_graph(|graph| graph.digraph()[self.id()].num_inputs())
    }

    /// Returns the number of outputs of the node.
    #[inline]
    pub fn num_outputs(&self) -> usize {
        self.graph
            .with_graph(|graph| graph.digraph()[self.id()].num_outputs())
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
            .with_graph(|graph| graph.digraph()[self.id()].input_spec()[index as usize].type_)
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
            .with_graph(|graph| graph.digraph()[self.id()].output_spec()[index as usize].type_)
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

        assert_eq!(
            output.output_type(source_output),
            self.input_type(target_input),
            "output and input signals must have the same type"
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

        assert_eq!(
            self.output_type(output_index),
            target.input_type(target_input),
            "output and input signals must have the same type"
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
    pub fn smooth(&self, factor: Float) -> Node {
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

    /// Connects a [`Len`] processor to the output of this node.
    ///
    /// # Panics
    ///
    /// - Panics if the node has multiple outputs.
    /// - Panics if the output signal is not a [`List`].
    #[inline]
    #[track_caller]
    pub fn len(&self) -> Node {
        self.assert_single_output("len");
        self.output(0).len()
    }

    /// Connects a [`Cast`] processor to the output of this node.
    ///
    /// The `type_` parameter specifies the type to cast the signal to.
    ///
    /// # Panics
    ///
    /// - Panics if the node has multiple outputs.
    /// - Panics if the output signal cannot be cast to the specified type.
    #[inline]
    #[track_caller]
    pub fn cast(&self, type_: SignalType) -> Node {
        self.assert_single_output("cast");
        self.output(0).cast(type_)
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
    pub fn type_(&self) -> SignalType {
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
        assert_eq!(
            self.type_(),
            output.type_(),
            "output and input signals must have the same type"
        );
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
    pub fn param<S: Signal>(
        &self,
        name: impl Into<String>,
        initial_value: impl Into<Option<S>>,
    ) -> Param<S> {
        let name = name.into();
        let param = Param::<S>::new(&name, initial_value);
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
    pub fn type_(&self) -> SignalType {
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
        assert_eq!(
            self.type_(),
            input.type_(),
            "output and input signals must have the same type"
        );
        self.node
            .connect_output(self.output_index, &input.node, input.input_index);
        self.node.clone()
    }

    /// Creates a [`Cast`] processor and connects it to the output.
    ///
    /// The `type_` parameter specifies the type to cast the signal to.
    ///
    /// # Panics
    ///
    /// Panics if the output signal cannot be cast to the specified type.
    #[inline]
    pub fn cast(&self, type_: SignalType) -> Node {
        let current_type = self.type_();
        if current_type == type_ {
            return self.node.clone();
        }
        let cast = match (current_type, type_) {
            // bool <-> int
            (SignalType::Bool, SignalType::Int) => self.node.graph().add(Cast::<bool, i64>::new()),
            (SignalType::Int, SignalType::Bool) => self.node.graph().add(Cast::<i64, bool>::new()),

            // bool <-> sample
            (SignalType::Bool, SignalType::Float) => {
                self.node.graph().add(Cast::<bool, Float>::new())
            }
            (SignalType::Float, SignalType::Bool) => {
                self.node.graph().add(Cast::<Float, bool>::new())
            }

            // int <-> sample
            (SignalType::Int, SignalType::Float) => {
                self.node.graph().add(Cast::<i64, Float>::new())
            }
            (SignalType::Float, SignalType::Int) => {
                self.node.graph().add(Cast::<Float, i64>::new())
            }

            // string <-> sample
            (SignalType::String, SignalType::Float) => {
                self.node.graph().add(Cast::<String, Float>::new())
            }
            (SignalType::Float, SignalType::String) => {
                self.node.graph().add(Cast::<Float, String>::new())
            }

            // string <-> int
            (SignalType::String, SignalType::Int) => {
                self.node.graph().add(Cast::<String, i64>::new())
            }
            (SignalType::Int, SignalType::String) => {
                self.node.graph().add(Cast::<i64, String>::new())
            }

            _ => panic!("cannot cast from {:?} to {:?}", current_type, type_),
        };

        cast.input(0).connect(self);
        cast
    }

    /// Creates a [`Passthrough`] processor and connects it to the output.
    ///
    /// This can be useful in situations where a [`Node`] is required instead of an [`Output`].
    #[inline]
    pub fn make_node(&self) -> Node {
        let type_ = self.type_();
        let node = match type_ {
            SignalType::Dynamic => self.node.graph().add(Passthrough::<AnySignal>::new()),
            SignalType::Bool => self.node.graph().add(Passthrough::<bool>::new()),
            SignalType::Int => self.node.graph().add(Passthrough::<i64>::new()),
            SignalType::Float => self.node.graph().add(Passthrough::<Float>::new()),
            SignalType::String => self.node.graph().add(Passthrough::<String>::new()),
            SignalType::List => self.node.graph().add(Passthrough::<List>::new()),
            SignalType::Midi => self.node.graph().add(Passthrough::<MidiMessage>::new()),
        };
        node.input(0).connect(self);
        node
    }

    /// Creates a [`Register`] processor and connects it to the output.
    ///
    /// The register processor stores the last value of the input signal and continuously outputs it.
    /// Useful for "remembering" a value across multiple frames.
    #[inline]
    pub fn make_register(&self) -> Node {
        let type_ = self.type_();
        let node = match type_ {
            SignalType::Dynamic => self.node.graph().add(Register::<AnySignal>::new()),
            SignalType::Bool => self.node.graph().add(Register::<bool>::new()),
            SignalType::Int => self.node.graph().add(Register::<i64>::new()),
            SignalType::Float => self.node.graph().add(Register::<Float>::new()),
            SignalType::String => self.node.graph().add(Register::<String>::new()),
            SignalType::List => self.node.graph().add(Register::<List>::new()),
            SignalType::Midi => self.node.graph().add(Register::<MidiMessage>::new()),
        };
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
        let type_ = then.type_();
        assert_eq!(
            type_,
            else_.type_(),
            "output signals must have the same type"
        );
        let cond = match type_ {
            SignalType::Dynamic => self.node.graph().add(Cond::<AnySignal>::new()),
            SignalType::Bool => self.node.graph().add(Cond::<bool>::new()),
            SignalType::Int => self.node.graph().add(Cond::<i64>::new()),
            SignalType::Float => self.node.graph().add(Cond::<Float>::new()),
            SignalType::String => self.node.graph().add(Cond::<String>::new()),
            SignalType::List => self.node.graph().add(Cond::<List>::new()),
            SignalType::Midi => self.node.graph().add(Cond::<MidiMessage>::new()),
        };
        cond.input("cond").connect(self);
        cond.input("then").connect(&then);
        cond.input("else").connect(&else_);
        cond
    }

    /// Creates a [`Len`] processor and connects it to the output.
    #[inline]
    pub fn len(&self) -> Node {
        assert_eq!(
            self.type_(),
            SignalType::List,
            "output signal must be a list"
        );
        let proc = self.node.graph().add(Len);
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
    impl<S: crate::signal::Signal> Sealed for crate::builtins::util::Param<S> {}
    impl Sealed for crate::signal::Float {}
    impl Sealed for i32 {}
    impl Sealed for i64 {}
    impl Sealed for u32 {}
    impl Sealed for &str {}
}

/// A trait for coercing a value into an [`Output`].
pub trait IntoOutput: sealed::Sealed {
    fn into_output(self, graph: &GraphBuilder) -> Output;
}

impl IntoOutput for Output {
    fn into_output(self, _graph: &GraphBuilder) -> Output {
        self
    }
}

impl IntoOutput for &Output {
    fn into_output(self, _graph: &GraphBuilder) -> Output {
        self.clone()
    }
}

impl<T: IntoNode> IntoOutput for T {
    fn into_output(self, graph: &GraphBuilder) -> Output {
        let node = self.into_node(graph);
        node.assert_single_output("into_output");
        node.output(0)
    }
}

/// A trait for coercing a value into a [`Node`].
pub trait IntoNode: sealed::Sealed {
    fn into_node(self, graph: &GraphBuilder) -> Node;
}

impl IntoNode for Node {
    fn into_node(self, graph: &GraphBuilder) -> Node {
        Node {
            graph: graph.clone(),
            node_id: self.node_id,
        }
    }
}

impl IntoNode for &Node {
    fn into_node(self, graph: &GraphBuilder) -> Node {
        Node {
            graph: graph.clone(),
            node_id: self.node_id,
        }
    }
}

impl<S: Signal> IntoNode for Param<S> {
    fn into_node(self, graph: &GraphBuilder) -> Node {
        graph.add(self)
    }
}

impl IntoNode for NodeIndex {
    fn into_node(self, graph: &GraphBuilder) -> Node {
        Node {
            graph: graph.clone(),
            node_id: self,
        }
    }
}

impl IntoNode for Float {
    fn into_node(self, graph: &GraphBuilder) -> Node {
        graph.constant(self)
    }
}

impl IntoNode for AnySignal {
    fn into_node(self, graph: &GraphBuilder) -> Node {
        graph.constant(self)
    }
}

impl IntoNode for i64 {
    fn into_node(self, graph: &GraphBuilder) -> Node {
        graph.constant(AnySignal::Int(self))
    }
}

impl IntoNode for i32 {
    fn into_node(self, graph: &GraphBuilder) -> Node {
        graph.constant(AnySignal::Int(self as i64))
    }
}

impl IntoNode for u32 {
    fn into_node(self, graph: &GraphBuilder) -> Node {
        graph.constant(AnySignal::Int(self as i64))
    }
}

impl IntoNode for &str {
    fn into_node(self, graph: &GraphBuilder) -> Node {
        graph.constant(AnySignal::String(self.to_string()))
    }
}

/// A trait for coercing a value into an output index of a node.
pub trait IntoOutputIdx: sealed::Sealed {
    fn into_output_idx(self, node: &Node) -> u32;
}

/// A trait for coercing a value into an input index of a node.
pub trait IntoInputIdx: sealed::Sealed {
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
        let Some(idx) = node.graph().with_graph(|graph| {
            graph.digraph()[node.id()]
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
        let Some(idx) = node.graph().with_graph(|graph| {
            graph.digraph()[node.id()]
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
    ($name:ident, $proc:ident, ($($type_:ident => $data:ty),*), $doc:literal) => {
        impl Output {
            #[allow(clippy::should_implement_trait)]
            #[doc = $doc]
            pub fn $name(&self, other: impl IntoOutput) -> Node {
                let other = other.into_output(self.node().graph());

                assert_eq!(
                    self.type_(),
                    other.type_(),
                    "output signals must have the same type"
                );

                let type_ = self.type_();
                let node = match type_ {
                    $(SignalType::$type_ => self.node().graph().add(<math::$proc<$data>>::default()),)*
                    _ => panic!("unsupported signal type for {:?}: {:?}", stringify!($name), type_),
                };

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
    ($name:ident, $std_op:ident, $proc:ident, ($($type_:ident => $data:ty),*), $doc:literal) => {
        impl Output {
            #[allow(clippy::should_implement_trait)]
            #[doc = $doc]
            pub fn $name(&self, other: impl IntoOutput) -> Node {
                let other = other.into_output(self.node().graph());

                assert_eq!(
                    self.type_(),
                    other.type_(),
                    "output signals must have the same type"
                );

                let type_ = self.type_();

                let node = match type_ {
                    $(SignalType::$type_ => self.node().graph().add(<math::$proc<$data>>::default()),)*
                    _ => panic!("unsupported signal type for {:?}: {:?}", stringify!($name), type_),
                };

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

impl_binary_node_ops!(add, Add, Add, (Float => Float, Int => i64), "Adds two signals together.");
impl_binary_node_ops!(sub, Sub, Sub, (Float => Float, Int => i64), "Subtracts one signal from another.");
impl_binary_node_ops!(mul, Mul, Mul, (Float => Float, Int => i64), "Multiplies two signals together.");
impl_binary_node_ops!(div, Div, Div, (Float => Float, Int => i64), "Divides one signal by another.");
impl_binary_node_ops!(
    rem,
    Rem,
    Rem,
    (Float => Float, Int => i64),
    "Calculates the remainder of one signal divided by another."
);
impl_binary_node_ops!(powf, Powf, (Float => Float), "Raises one signal to the power of another.");
impl_binary_node_ops!(
    atan2,
    Atan2,
    (Float => Float),
    "Calculates the arctangent of the ratio of two signals."
);
impl_binary_node_ops!(hypot, Hypot, (Float => Float), "Calculates the hypotenuse of two signals.");
impl_binary_node_ops!(max, Max, (Float => Float, Int => i64), "Outputs the maximum of two signals.");
impl_binary_node_ops!(min, Min, (Float => Float, Int => i64), "Outputs the minimum of two signals.");

macro_rules! impl_comparison_node_ops {
    ($name:ident, $proc:ident, $doc:expr) => {
        impl Output {
            #[allow(clippy::should_implement_trait)]
            #[doc = $doc]
            pub fn $name(&self, other: impl IntoOutput) -> Node {
                let other = other.into_output(self.node().graph());

                assert_eq!(
                    self.type_(),
                    other.type_(),
                    "output signals must have the same type"
                );

                let type_ = self.type_();

                let node = match type_ {
                    SignalType::Dynamic => {
                        self.node().graph().add(control::$proc::<AnySignal>::new())
                    }
                    SignalType::Bool => self.node().graph().add(control::$proc::<bool>::default()),
                    SignalType::Int => self.node().graph().add(control::$proc::<i64>::default()),
                    SignalType::Float => {
                        self.node().graph().add(control::$proc::<Float>::default())
                    }
                    SignalType::String => {
                        self.node().graph().add(control::$proc::<String>::default())
                    }
                    _ => panic!("unsupported signal type"),
                };

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
    ($name:ident, $proc:ident, ($($type_:ident => $data:ty),*), $doc:literal) => {
        impl Output {
            #[allow(clippy::should_implement_trait)]
            #[doc = $doc]
            pub fn $name(&self) -> Node {
                let type_ = self.type_();

                let node = match type_ {
                    $(SignalType::$type_ => self.node().graph().add(<math::$proc<$data>>::default()),)*
                    _ => panic!("unsupported signal type for {:?}: {:?}", stringify!($name), type_),
                };

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

impl_unary_node_ops!(neg, Neg, (Float => Float, Int => i64), "Negates the input signal.");

impl std::ops::Neg for &Node {
    type Output = Node;

    fn neg(self) -> Node {
        Node::neg(self)
    }
}

impl_unary_node_ops!(
    abs,
    Abs,
    (Float => Float, Int => i64),
    "Outputs the absolute value of the input signal."
);
impl_unary_node_ops!(
    sqrt,
    Sqrt,
    (Float => Float),
    "Outputs the square root of the input signal."
);
impl_unary_node_ops!(
    cbrt,
    Cbrt,
    (Float => Float),
    "Outputs the cube root of the input signal."
);
impl_unary_node_ops!(
    ceil,
    Ceil,
    (Float => Float),
    "Rounds the input signal up to the nearest integer."
);
impl_unary_node_ops!(
    floor,
    Floor,
    (Float => Float),
    "Rounds the input signal down to the nearest integer."
);
impl_unary_node_ops!(
    round,
    Round,
    (Float => Float),
    "Rounds the input signal to the nearest integer."
);
impl_unary_node_ops!(sin, Sin, (Float => Float), "Outputs the sine of the input signal.");
impl_unary_node_ops!(cos, Cos, (Float => Float), "Outputs the cosine of the input signal.");
impl_unary_node_ops!(tan, Tan, (Float => Float), "Outputs the tangent of the input signal.");
impl_unary_node_ops!(
    tanh,
    Tanh,
    (Float => Float),
    "Outputs the hyperbolic tangent of the input signal."
);

impl_unary_node_ops!(
    recip,
    Recip,
    (Float => Float),
    "Outputs the reciprocal of the input signal."
);
impl_unary_node_ops!(
    signum,
    Signum,
    (Float => Float, Int => i64),
    "Outputs the sign of the input signal."
);
impl_unary_node_ops!(
    fract,
    Fract,
    (Float => Float),
    "Outputs the fractional part of the input signal."
);
impl_unary_node_ops!(
    ln,
    Ln,
    (Float => Float),
    "Outputs the natural logarithm of the input signal."
);
impl_unary_node_ops!(
    log2,
    Log2,
    (Float => Float),
    "Outputs the base-2 logarithm of the input signal."
);
impl_unary_node_ops!(
    log10,
    Log10,
    (Float => Float),
    "Outputs the base-10 logarithm of the input signal."
);
impl_unary_node_ops!(
    exp,
    Exp,
    (Float => Float),
    "Outputs the natural exponential of the input signal."
);
