//! Contains the [`ProcessorNode`] and [`Node`] structs, which represent nodes in the audio graph that process signals.

use std::{fmt::Debug, ops::Deref};

use raug_graph::node::Node as AbstractNode;
use thiserror::Error;

use crate::{prelude::*, signal::type_erased::AnyBuffer};

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
    pub(crate) processor: Box<dyn Processor>,
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
            processor: Box::new(processor),
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
        self.processor.allocate(sample_rate, max_block_size);
        self.outputs = self.processor.create_output_buffers(max_block_size);
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
        inputs: &[Option<&AnyBuffer>],
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
        if let Err(e) = self.processor.process(inputs, outputs) {
            return Err(ProcessNodeError {
                error: e,
                node_name: self.name().to_string(),
            });
        }

        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct Output<O: AsNodeOutputIndex<ProcessorNode>>(pub(crate) NodeOutput<ProcessorNode, O>);

impl<O: AsNodeOutputIndex<ProcessorNode>> Deref for Output<O> {
    type Target = NodeOutput<ProcessorNode, O>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Copy)]
pub struct Input<I: AsNodeInputIndex<ProcessorNode>>(pub(crate) NodeInput<ProcessorNode, I>);

impl<I: AsNodeInputIndex<ProcessorNode>> Deref for Input<I> {
    type Target = NodeInput<ProcessorNode, I>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait IntoNodeOutput<O: AsNodeOutputIndex<ProcessorNode>>: Send + 'static {
    fn into_node_output(self, graph: &mut Graph) -> Output<O>;
}

impl<O: AsNodeOutputIndex<ProcessorNode>> IntoNodeOutput<O> for Output<O> {
    fn into_node_output(self, _graph: &mut Graph) -> Output<O> {
        self
    }
}

impl<S: Signal + Default + Clone + Copy> IntoNodeOutput<u32> for S {
    fn into_node_output(self, graph: &mut Graph) -> Output<u32> {
        let node = graph.constant(self);
        Output(NodeOutput::new(node.into(), 0))
    }
}

impl IntoNodeOutput<u32> for Node {
    fn into_node_output(self, _graph: &mut Graph) -> Output<u32> {
        Output(NodeOutput::new(self.into(), 0))
    }
}

pub trait RaugNodeIndexExt {
    fn output<O: AsNodeOutputIndex<ProcessorNode>>(&self, index: O) -> Output<O>;
    fn input<I: AsNodeInputIndex<ProcessorNode>>(&self, index: I) -> Input<I>;
}

impl RaugNodeIndexExt for Node {
    fn output<O: AsNodeOutputIndex<ProcessorNode>>(&self, index: O) -> Output<O> {
        Output(NodeOutput::new(self.0, index))
    }

    fn input<I: AsNodeInputIndex<ProcessorNode>>(&self, index: I) -> Input<I> {
        Input(NodeInput::new(self.0, index))
    }
}

pub trait BuildOnGraph: Send + 'static {
    fn build_on_graph(self, graph: &mut Graph) -> Node;
}

impl BuildOnGraph for Node {
    fn build_on_graph(self, _graph: &mut Graph) -> Node {
        // todo: check if node exists in graph
        self
    }
}

impl<P: Processor> BuildOnGraph for P {
    fn build_on_graph(self, graph: &mut Graph) -> Node {
        graph.processor(self)
    }
}

impl BuildOnGraph for &'static str {
    fn build_on_graph(self, graph: &mut Graph) -> Node {
        graph.constant(Str::from(self))
    }
}

impl BuildOnGraph for f32 {
    fn build_on_graph(self, graph: &mut Graph) -> Node {
        graph.constant(self)
    }
}

pub struct NodeBinaryOp<A, B, Op, O1, O2>
where
    A: IntoNodeOutput<O1>,
    B: IntoNodeOutput<O2>,
    Op: Processor + Default,
    O1: AsNodeOutputIndex<ProcessorNode>,
    O2: AsNodeOutputIndex<ProcessorNode>,
{
    pub a: A,
    pub op: Op,
    pub b: B,
    _o1: std::marker::PhantomData<O1>,
    _o2: std::marker::PhantomData<O2>,
}

impl<A, B, Op, O1, O2> NodeBinaryOp<A, B, Op, O1, O2>
where
    A: IntoNodeOutput<O1>,
    B: IntoNodeOutput<O2>,
    Op: Processor + Default,
    O1: AsNodeOutputIndex<ProcessorNode>,
    O2: AsNodeOutputIndex<ProcessorNode>,
{
    pub fn new(a: A, op: Op, b: B) -> Self {
        Self {
            a,
            op,
            b,
            _o1: std::marker::PhantomData,
            _o2: std::marker::PhantomData,
        }
    }
}

impl<A, B, Op, O1, O2> BuildOnGraph for NodeBinaryOp<A, B, Op, O1, O2>
where
    A: IntoNodeOutput<O1>,
    B: IntoNodeOutput<O2>,
    Op: Processor + Default,
    O1: AsNodeOutputIndex<ProcessorNode>,
    O2: AsNodeOutputIndex<ProcessorNode>,
{
    fn build_on_graph(self, graph: &mut Graph) -> Node {
        graph.bin_op(self.a, self.op, self.b)
    }
}

impl<A, B, Op, O1, O2> IntoNodeOutput<u32> for NodeBinaryOp<A, B, Op, O1, O2>
where
    A: IntoNodeOutput<O1>,
    B: IntoNodeOutput<O2>,
    Op: Processor + Default,
    O1: AsNodeOutputIndex<ProcessorNode>,
    O2: AsNodeOutputIndex<ProcessorNode>,
{
    fn into_node_output(self, graph: &mut Graph) -> Output<u32> {
        let node = self.build_on_graph(graph);
        node.output(0)
    }
}

macro_rules! impl_node_binary_op {
    ($op:ident $func:ident) => {
        // output<u32> op output<u32>
        impl std::ops::$op<Output<u32>> for Output<u32> {
            type Output = NodeBinaryOp<Self, Output<u32>, $op, u32, u32>;

            fn $func(self, rhs: Output<u32>) -> Self::Output {
                NodeBinaryOp::new(self, $op::default(), rhs)
            }
        }

        // output<&str> op output<u32>
        impl std::ops::$op<Output<&'static str>> for Output<u32> {
            type Output = NodeBinaryOp<Self, Output<&'static str>, $op, u32, &'static str>;

            fn $func(self, rhs: Output<&'static str>) -> Self::Output {
                NodeBinaryOp::new(self, $op::default(), rhs)
            }
        }

        // output<u32> op output<&str>
        impl std::ops::$op<Output<u32>> for Output<&'static str> {
            type Output = NodeBinaryOp<Self, Output<u32>, $op, &'static str, u32>;

            fn $func(self, rhs: Output<u32>) -> Self::Output {
                NodeBinaryOp::new(self, $op::default(), rhs)
            }
        }

        // output<&str> op output<&str>
        impl std::ops::$op<Output<&'static str>> for Output<&'static str> {
            type Output = NodeBinaryOp<Self, Output<&'static str>, $op, &'static str, &'static str>;

            fn $func(self, rhs: Output<&'static str>) -> Self::Output {
                NodeBinaryOp::new(self, $op::default(), rhs)
            }
        }

        // node op node
        impl std::ops::$op for Node {
            type Output = NodeBinaryOp<Output<u32>, Output<u32>, $op, u32, u32>;

            fn $func(self, rhs: Self) -> Self::Output {
                NodeBinaryOp::new(self.output(0), $op::default(), rhs.output(0))
            }
        }

        // output<u32> op f32
        impl std::ops::$op<f32> for Output<u32> {
            type Output = NodeBinaryOp<Output<u32>, f32, $op, u32, u32>;

            fn $func(self, rhs: f32) -> Self::Output {
                NodeBinaryOp::new(self, $op::default(), rhs)
            }
        }

        // f32 op output<u32>
        impl std::ops::$op<Output<u32>> for f32 {
            type Output = NodeBinaryOp<f32, Output<u32>, $op, u32, u32>;

            fn $func(self, rhs: Output<u32>) -> Self::Output {
                NodeBinaryOp::new(self, $op::default(), rhs)
            }
        }

        // node op output<u32>
        impl std::ops::$op<Output<u32>> for Node {
            type Output = NodeBinaryOp<Node, Output<u32>, $op, u32, u32>;

            fn $func(self, rhs: Output<u32>) -> Self::Output {
                NodeBinaryOp::new(self, $op::default(), rhs)
            }
        }

        // output<u32> op node
        impl std::ops::$op<Node> for Output<u32> {
            type Output = NodeBinaryOp<Output<u32>, Node, $op, u32, u32>;

            fn $func(self, rhs: Node) -> Self::Output {
                NodeBinaryOp::new(self, $op::default(), rhs)
            }
        }

        // node op f32
        impl std::ops::$op<f32> for Node {
            type Output = NodeBinaryOp<Node, f32, $op, u32, u32>;

            fn $func(self, rhs: f32) -> Self::Output {
                NodeBinaryOp::new(self, $op::default(), rhs)
            }
        }

        // f32 op node
        impl std::ops::$op<Node> for f32 {
            type Output = NodeBinaryOp<f32, Node, $op, u32, u32>;

            fn $func(self, rhs: Node) -> Self::Output {
                NodeBinaryOp::new(self, $op::default(), rhs)
            }
        }

        // node op op2
        impl<A2, B2, Op2, O1, O2> std::ops::$op<NodeBinaryOp<A2, B2, Op2, O1, O2>> for Node
        where
            B2: IntoNodeOutput<O2>,
            A2: IntoNodeOutput<O1>,
            B2: IntoNodeOutput<O2>,
            Op2: Processor + Default,
            O1: AsNodeOutputIndex<ProcessorNode>,
            O2: AsNodeOutputIndex<ProcessorNode>,
        {
            type Output = NodeBinaryOp<Self, NodeBinaryOp<A2, B2, Op2, O1, O2>, $op, u32, u32>;

            fn $func(self, rhs: NodeBinaryOp<A2, B2, Op2, O1, O2>) -> Self::Output {
                NodeBinaryOp::new(self, $op::default(), rhs)
            }
        }

        // op1 op node
        impl<A, B, Op, O1, O2> std::ops::$op<Node> for NodeBinaryOp<A, B, Op, O1, O2>
        where
            A: IntoNodeOutput<O1>,
            B: IntoNodeOutput<O2>,
            Op: Processor + Default,
            O1: AsNodeOutputIndex<ProcessorNode>,
            O2: AsNodeOutputIndex<ProcessorNode>,
        {
            type Output = NodeBinaryOp<Self, Node, $op, u32, u32>;

            fn $func(self, rhs: Node) -> Self::Output {
                NodeBinaryOp::new(self, $op::default(), rhs)
            }
        }

        // op1 op op2
        impl<A, B, Op1, Op2, O1, O2> std::ops::$op<NodeBinaryOp<A, B, Op1, O1, O2>>
            for NodeBinaryOp<A, B, Op2, O1, O2>
        where
            A: IntoNodeOutput<O1>,
            B: IntoNodeOutput<O2>,
            Op1: Processor + Default,
            Op2: Processor + Default,
            O1: AsNodeOutputIndex<ProcessorNode>,
            O2: AsNodeOutputIndex<ProcessorNode>,
        {
            type Output = NodeBinaryOp<Self, NodeBinaryOp<A, B, Op1, O1, O2>, $op, u32, u32>;

            fn $func(self, rhs: NodeBinaryOp<A, B, Op1, O1, O2>) -> Self::Output {
                NodeBinaryOp::new(self, $op::default(), rhs)
            }
        }
    };
}

impl_node_binary_op!(Add add);
impl_node_binary_op!(Sub sub);
impl_node_binary_op!(Mul mul);
impl_node_binary_op!(Div div);
impl_node_binary_op!(Rem rem);
