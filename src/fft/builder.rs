//! Builder API for constructing [`FftGraph`]s.

use std::sync::{Arc, Mutex};

use petgraph::prelude::*;

use crate::{fft_builtins::*, prelude::*};

/// A builder API version of an [`FftGraph`].
#[derive(Clone)]
pub struct FftGraphBuilder {
    graph: Arc<Mutex<FftGraph>>,
}

impl Default for FftGraphBuilder {
    fn default() -> Self {
        Self::new(128, 64, WindowFunction::Hann)
    }
}

impl FftGraphBuilder {
    /// Creates a new `FftGraphBuilder` with the given parameters.
    pub fn new(fft_length: usize, hop_length: usize, window_function: WindowFunction) -> Self {
        Self {
            graph: Arc::new(Mutex::new(FftGraph::new(
                fft_length,
                hop_length,
                window_function,
            ))),
        }
    }

    /// Creates a new `FftGraphBuilder` from an existing [`FftGraph`].
    pub fn from_graph(graph: FftGraph) -> Self {
        Self {
            graph: Arc::new(Mutex::new(graph)),
        }
    }

    /// Executes the given closure with a mutable reference to the underlying [`FftGraph`].
    pub fn with_graph<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut FftGraph) -> R,
    {
        let mut graph = self.graph.lock().unwrap();
        f(&mut graph)
    }

    /// Returns a clone of the underlying [`FftGraph`] as it currently exists.
    pub fn build(&self) -> FftGraph {
        self.with_graph(|graph| graph.clone())
    }

    /// Adds an audio input node to the graph.
    pub fn add_audio_input(&self) -> FftNode {
        let node_id = self.with_graph(|graph| graph.add_audio_input());
        FftNode {
            node_id,
            graph: self.clone(),
        }
    }

    /// Adds an audio output node to the graph.
    pub fn add_audio_output(&self) -> FftNode {
        let node_id = self.with_graph(|graph| graph.add_audio_output());
        FftNode {
            node_id,
            graph: self.clone(),
        }
    }

    /// Adds a processor node to the graph.
    pub fn add(&self, processor: impl FftProcessor) -> FftNode {
        let node_id = self.with_graph(|graph| graph.add(processor));
        FftNode {
            node_id,
            graph: self.clone(),
        }
    }

    /// Connects the given output of one node to the given input of another node.
    ///
    /// If a connection already exists at the given input index, it will be replaced.
    pub fn connect(
        &self,
        source: &FftNode,
        source_output: usize,
        target: &FftNode,
        target_input: usize,
    ) {
        self.with_graph(|graph| {
            graph.connect(source.id(), source_output, target.id(), target_input)
        });
    }

    pub fn real_splat(&self, value: f32, len: FftBufLength) -> FftNode {
        self.add(RealSplat::new(value, len))
    }

    #[track_caller]
    pub fn car2pol(&self, real: &FftNode, imag: &FftNode) -> (FftNode, FftNode) {
        assert_eq!(real.output_spec().len(), 1);
        assert_eq!(imag.output_spec().len(), 1);
        assert!(matches!(
            real.output_spec()[0].signal_type,
            FftSignalType::RealBuf(_)
        ));
        assert!(matches!(
            imag.output_spec()[0].signal_type,
            FftSignalType::RealBuf(_)
        ));
        let node = self.add(ComplexToPolar::default());
        self.connect(&real, 0, &node, 0);
        self.connect(&imag, 0, &node, 1);
        let mag = node.output(0).make_node();
        let phase = node.output(1).make_node();
        (mag, phase)
    }

    #[track_caller]
    pub fn pol2car(&self, mag: &FftNode, phase: &FftNode) -> FftNode {
        assert_eq!(mag.output_spec().len(), 1);
        assert_eq!(phase.output_spec().len(), 1);
        assert!(matches!(
            mag.output_spec()[0].signal_type,
            FftSignalType::RealBuf(_)
        ));
        assert!(matches!(
            phase.output_spec()[0].signal_type,
            FftSignalType::RealBuf(_)
        ));
        let node = self.add(ComplexFromPolar);
        self.connect(&mag, 0, &node, 0);
        self.connect(&phase, 0, &node, 1);
        node
    }
}

/// A node in an [`FftGraphBuilder`].
#[derive(Clone)]
pub struct FftNode {
    node_id: NodeIndex,
    graph: FftGraphBuilder,
}

impl FftNode {
    /// Returns the ID of the node.
    pub fn id(&self) -> NodeIndex {
        self.node_id
    }

    /// Returns the [`FftGraphBuilder`] that the node belongs to.
    pub fn graph(&self) -> FftGraphBuilder {
        self.graph.clone()
    }

    /// Returns an [`FftInput`] for the given index, allowing further operations on that input.
    pub fn input(&self, index: impl IntoFftInput) -> FftInput {
        index.into_fft_input(self)
    }

    /// Returns an [`FftOutput`] for the given index, allowing further operations on that output.
    pub fn output(&self, index: impl IntoFftOutput) -> FftOutput {
        index.into_fft_output(self)
    }

    pub fn input_spec(&self) -> Vec<FftSpec> {
        self.graph.with_graph(|graph| {
            graph
                .digraph
                .node_weight(self.node_id)
                .unwrap()
                .input_spec()
                .to_vec()
        })
    }

    pub fn output_spec(&self) -> Vec<FftSpec> {
        self.graph.with_graph(|graph| {
            graph
                .digraph
                .node_weight(self.node_id)
                .unwrap()
                .output_spec()
                .to_vec()
        })
    }
}

#[doc(hidden)]
mod sealed {
    use super::*;
    pub trait Sealed {}
    impl Sealed for FftNode {}
    impl Sealed for &FftNode {}
    impl Sealed for FftInput {}
    impl Sealed for &FftInput {}
    impl Sealed for FftOutput {}
    impl Sealed for &FftOutput {}
    impl Sealed for u32 {}
    impl Sealed for i32 {}
    impl Sealed for usize {}
    impl Sealed for f32 {}
    impl Sealed for &str {}
}

#[doc(hidden)]
pub trait IntoFftNode: sealed::Sealed {
    fn into_fft_node(self, graph: &FftGraphBuilder) -> FftNode;
}

impl IntoFftNode for FftNode {
    fn into_fft_node(self, _: &FftGraphBuilder) -> FftNode {
        self
    }
}

impl IntoFftNode for &FftNode {
    fn into_fft_node(self, _: &FftGraphBuilder) -> FftNode {
        self.clone()
    }
}

#[doc(hidden)]
pub trait IntoFftInput: sealed::Sealed {
    fn into_fft_input(self, node: &FftNode) -> FftInput;
}

impl IntoFftInput for u32 {
    fn into_fft_input(self, node: &FftNode) -> FftInput {
        FftInput {
            node: node.clone(),
            index: self as usize,
        }
    }
}

impl IntoFftInput for i32 {
    fn into_fft_input(self, node: &FftNode) -> FftInput {
        FftInput {
            node: node.clone(),
            index: self as usize,
        }
    }
}

impl IntoFftInput for usize {
    fn into_fft_input(self, node: &FftNode) -> FftInput {
        FftInput {
            node: node.clone(),
            index: self,
        }
    }
}

impl IntoFftInput for &str {
    #[track_caller]
    fn into_fft_input(self, node: &FftNode) -> FftInput {
        let index = node
            .input_spec()
            .iter()
            .position(|spec| spec.name == *self)
            .unwrap_or_else(|| panic!("input not found: {}", self));
        FftInput {
            node: node.clone(),
            index,
        }
    }
}

#[doc(hidden)]
pub trait IntoFftOutput: sealed::Sealed {
    fn into_fft_output(self, node: &FftNode) -> FftOutput;
}

impl IntoFftOutput for u32 {
    fn into_fft_output(self, node: &FftNode) -> FftOutput {
        FftOutput {
            node: node.clone(),
            index: self as usize,
        }
    }
}

impl IntoFftOutput for i32 {
    fn into_fft_output(self, node: &FftNode) -> FftOutput {
        FftOutput {
            node: node.clone(),
            index: self as usize,
        }
    }
}

impl IntoFftOutput for &str {
    #[track_caller]
    fn into_fft_output(self, node: &FftNode) -> FftOutput {
        let index = node
            .output_spec()
            .iter()
            .position(|spec| spec.name == *self)
            .unwrap_or_else(|| panic!("output not found: {}", self));
        FftOutput {
            node: node.clone(),
            index,
        }
    }
}

impl IntoFftOutput for usize {
    fn into_fft_output(self, node: &FftNode) -> FftOutput {
        FftOutput {
            node: node.clone(),
            index: self,
        }
    }
}

impl IntoFftOutput for FftOutput {
    fn into_fft_output(self, _: &FftNode) -> FftOutput {
        self
    }
}

impl IntoFftOutput for &FftOutput {
    fn into_fft_output(self, _: &FftNode) -> FftOutput {
        self.clone()
    }
}

impl IntoFftOutput for FftNode {
    #[track_caller]
    fn into_fft_output(self, _: &FftNode) -> FftOutput {
        assert_eq!(self.output_spec().len(), 1);
        self.output(0)
    }
}

impl IntoFftOutput for &FftNode {
    #[track_caller]
    fn into_fft_output(self, _: &FftNode) -> FftOutput {
        assert_eq!(self.output_spec().len(), 1);
        self.output(0)
    }
}

/// An input to an [`FftNode`].
#[derive(Clone)]
pub struct FftInput {
    node: FftNode,
    index: usize,
}

impl FftInput {
    /// Returns the [`FftNode`] that the input belongs to.
    pub fn node(&self) -> FftNode {
        self.node.clone()
    }

    /// Returns the index of the input.
    pub fn index(&self) -> usize {
        self.index
    }

    pub fn signal_type(&self) -> FftSignalType {
        self.node.input_spec()[self.index].signal_type
    }

    /// Connects the input to the given output.
    #[track_caller]
    pub fn connect(&self, output: impl IntoFftOutput) {
        let output = output.into_fft_output(&self.node);
        assert_eq!(self.signal_type(), output.signal_type());
        self.node.graph.with_graph(|graph| {
            graph.connect(output.node.id(), output.index, self.node.id(), self.index)
        });
    }
}

/// An output of an [`FftNode`].
#[derive(Clone)]
pub struct FftOutput {
    node: FftNode,
    index: usize,
}

impl FftOutput {
    /// Returns the [`FftNode`] that the output belongs to.
    pub fn node(&self) -> FftNode {
        self.node.clone()
    }

    /// Returns the index of the output.
    pub fn index(&self) -> usize {
        self.index
    }

    pub fn signal_type(&self) -> FftSignalType {
        self.node.output_spec()[self.index].signal_type
    }

    /// Connects the output to the given input.
    #[track_caller]
    pub fn connect(&self, input: &FftInput) {
        assert_eq!(self.signal_type(), input.signal_type());
        self.node.graph.with_graph(|graph| {
            graph.connect(self.node.id(), self.index, input.node.id(), input.index)
        });
    }

    pub fn make_node(&self) -> FftNode {
        let graph = self.node.graph();
        let proc = match self.signal_type() {
            FftSignalType::Param(_) => return self.node.clone(),
            FftSignalType::RealBuf(len) => graph.add(RealBufPassthrough(len)),
            FftSignalType::ComplexBuf(len) => graph.add(ComplexBufPassthrough(len)),
        };
        graph.connect(&self.node, self.index, &proc, 0);
        proc
    }

    #[track_caller]
    pub fn car2pol(&self) -> (FftNode, FftNode) {
        assert!(matches!(self.signal_type(), FftSignalType::ComplexBuf(_)));
        let graph = self.node.graph();
        let node = graph.add(ComplexToPolar);
        graph.connect(&self.node, self.index, &node, 0);
        let mag = node.output(0).make_node();
        let phase = node.output(1).make_node();
        (mag, phase)
    }

    #[track_caller]
    pub fn conj(&self) -> FftNode {
        assert!(matches!(self.signal_type(), FftSignalType::ComplexBuf(_)));
        let graph = self.node.graph();
        let node = graph.add(ComplexConjugate);
        node.input(0).connect(self);
        node
    }

    pub fn print(&self) -> FftNode {
        let graph = self.node.graph();
        let node = graph.add(FftPrint(self.signal_type()));
        node.input(0).connect(self);
        node
    }
}

macro_rules! fft_node_binary_op {
    ($real_proc:ident, $complex_proc:ident, $std_op:ident, $std_op_fn:ident) => {
        impl<T: IntoFftOutput> std::ops::$std_op<T> for FftOutput {
            type Output = FftNode;

            #[track_caller]
            fn $std_op_fn(self, other: T) -> FftNode {
                let other = other.into_fft_output(&self.node);
                let graph = self.node.graph();
                let node = match self.signal_type() {
                    FftSignalType::Param(_) => self.node.clone(),
                    FftSignalType::RealBuf(len) => graph.add($real_proc(len)),
                    FftSignalType::ComplexBuf(len) => graph.add($complex_proc(len)),
                };
                self.connect(&node.input(0));
                other.connect(&node.input(1));
                node
            }
        }

        impl<T: IntoFftOutput> std::ops::$std_op<T> for FftNode {
            type Output = FftNode;

            #[track_caller]
            fn $std_op_fn(self, other: T) -> FftNode {
                assert_eq!(self.output_spec().len(), 1);
                let other = other.into_fft_output(&self);
                let graph = self.graph();
                let node = match self.output_spec()[0].signal_type {
                    FftSignalType::Param(_) => self.clone(),
                    FftSignalType::RealBuf(len) => graph.add($real_proc(len)),
                    FftSignalType::ComplexBuf(len) => graph.add($complex_proc(len)),
                };
                self.output(0).connect(&node.input(0));
                other.connect(&node.input(1));
                node
            }
        }

        impl<T: IntoFftOutput> std::ops::$std_op<T> for &FftNode {
            type Output = FftNode;

            #[track_caller]
            fn $std_op_fn(self, other: T) -> FftNode {
                assert_eq!(self.output_spec().len(), 1);
                let other = other.into_fft_output(&self);
                let graph = self.graph();
                let node = match self.output_spec()[0].signal_type {
                    FftSignalType::Param(_) => self.clone(),
                    FftSignalType::RealBuf(len) => graph.add($real_proc(len)),
                    FftSignalType::ComplexBuf(len) => graph.add($complex_proc(len)),
                };
                self.output(0).connect(&node.input(0));
                other.connect(&node.input(1));
                node
            }
        }
    };
}

fft_node_binary_op!(RealAdd, ComplexAdd, Add, add);
fft_node_binary_op!(RealSub, ComplexSub, Sub, sub);
fft_node_binary_op!(RealMul, ComplexMul, Mul, mul);
fft_node_binary_op!(RealDiv, ComplexDiv, Div, div);
fft_node_binary_op!(RealRem, ComplexRem, Rem, rem);
