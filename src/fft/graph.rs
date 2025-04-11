//! A directed graph of nodes that process FFT signals.

use std::collections::{BTreeMap, VecDeque};

use num::Complex;
use petgraph::prelude::*;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{fft_builtins::*, prelude::*};

/// A node in an [`FftGraph`] that processes FFT signals.
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FftProcessorNode {
    processor: Box<dyn FftProcessor>,
    input_spec: Vec<FftSpec>,
    output_spec: Vec<FftSpec>,
}

impl FftProcessorNode {
    /// Creates a new `FftProcessorNode` with the given [`FftProcessor`].
    pub fn new(processor: impl FftProcessor) -> Self {
        Self::new_boxed(Box::new(processor))
    }

    /// Creates a new `FftProcessorNode` with the given boxed [`FftProcessor`].
    pub fn new_boxed(processor: Box<dyn FftProcessor>) -> Self {
        let input_spec = processor.input_spec();
        let output_spec = processor.output_spec();
        Self {
            processor,
            input_spec,
            output_spec,
        }
    }

    /// Returns information about the input signals of the processor.
    pub fn input_spec(&self) -> &[FftSpec] {
        &self.input_spec
    }

    /// Returns information about the output signals of the processor.
    pub fn output_spec(&self) -> &[FftSpec] {
        &self.output_spec
    }

    /// Allocates memory for the processor.
    pub fn allocate(&mut self, fft_length: usize, padded_length: usize) {
        self.processor.allocate(fft_length, padded_length);
    }

    pub fn process(
        &mut self,
        fft_length: usize,
        inputs: &[&FftSignal],
        outputs: &mut [FftSignal],
    ) -> Result<(), ProcessorError> {
        self.processor.process(fft_length, inputs, outputs)
    }
}

/// A connection between two nodes in an [`FftGraph`].
#[derive(Clone, Debug, Default, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FftEdge {
    /// The index of the output signal of the source node.
    pub source_output: usize,
    /// The index of the input signal of the target node.
    pub target_input: usize,
}

type FftGraphVisitor = DfsPostOrder<NodeIndex, FxHashSet<NodeIndex>>;

#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FftAudioInput {
    pub(crate) ring_buffer: VecDeque<f32>,
    pub(crate) time_domain: FftSignal,
}

impl Default for FftAudioInput {
    fn default() -> Self {
        Self {
            ring_buffer: VecDeque::new(),
            time_domain: FftSignal::RealBuf(RealBuf::default()),
        }
    }
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FftAudioOutput {
    pub(crate) ring_buffer: VecDeque<f32>,
    pub(crate) overlap_buffer: VecDeque<f32>,
}

/// A directed graph of nodes that process FFT signals.
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FftGraph {
    pub(crate) digraph: StableDiGraph<FftProcessorNode, FftEdge>,

    fft_length: usize,
    hop_length: usize,
    #[allow(unused)]
    window_function: WindowFunction,
    window: RealBuf,

    inputs: Vec<NodeIndex>,
    outputs: Vec<NodeIndex>,
    audio_inputs: FxHashMap<NodeIndex, FftAudioInput>,
    audio_outputs: FxHashMap<NodeIndex, FftAudioOutput>,

    #[cfg_attr(feature = "serde", serde(skip))]
    visitor: FftGraphVisitor,
    visit_path: Vec<NodeIndex>,

    buffer_cache: FxHashMap<NodeIndex, Vec<FftSignal>>,
}

impl Default for FftGraph {
    fn default() -> Self {
        Self::new(256, 64, WindowFunction::Hann)
    }
}

impl FftGraph {
    /// Creates a new, empty `FftGraph` with the given FFT length, hop length, and window function.
    pub fn new(fft_length: usize, hop_length: usize, window_function: WindowFunction) -> Self {
        let mut window = window_function.generate(fft_length);

        // rotate the window 180 degrees, so it is centered around 0
        window.rotate_right(fft_length / 2);

        let overlapping_frames = fft_length / hop_length;
        let mut window_sum = window.iter().sum::<f32>();
        window_sum *= 2.0 * overlapping_frames as f32;

        // normalize the window
        for x in window.iter_mut() {
            *x /= window_sum;
        }

        Self {
            fft_length,
            hop_length,
            window,
            window_function,
            digraph: StableDiGraph::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            audio_inputs: FxHashMap::default(),
            audio_outputs: FxHashMap::default(),
            visitor: FftGraphVisitor::default(),
            visit_path: Vec::new(),
            buffer_cache: FxHashMap::default(),
        }
    }

    /// Constructs an `FftGraph` with a closure and the [`FftGraphBuilder`] API.
    pub fn build(self, f: impl FnOnce(&mut FftGraphBuilder)) -> Self {
        let mut builder = FftGraphBuilder::from_graph(self);
        f(&mut builder);
        builder.build()
    }

    /// Returns the FFT window length of the graph (how many FFT points are used).
    pub fn fft_length(&self) -> usize {
        self.fft_length
    }

    /// Returns the hop length of the graph (the stride between FFT frames).
    pub fn hop_length(&self) -> usize {
        self.hop_length
    }

    /// Returns the overlap length of the graph (how many samples overlap between FFT frames).
    pub fn overlap_length(&self) -> usize {
        self.fft_length - self.hop_length
    }

    /// Adds an input node to the graph and returns its index.
    pub fn add_input(&mut self, processor: impl FftProcessor) -> NodeIndex {
        let node = self.digraph.add_node(FftProcessorNode::new(processor));
        self.inputs.push(node);
        node
    }

    pub fn add_audio_input(&mut self) -> NodeIndex {
        let index = self.add_input(Rfft::new(self.fft_length * 2));
        self.audio_inputs.insert(index, FftAudioInput::default());
        index
    }

    /// Adds an output node to the graph and returns its index.
    pub fn add_output(&mut self, processor: impl FftProcessor) -> NodeIndex {
        let node = self.digraph.add_node(FftProcessorNode::new(processor));
        self.outputs.push(node);
        node
    }

    pub fn add_audio_output(&mut self) -> NodeIndex {
        let index = self.add_output(Irfft::new(self.fft_length * 2));
        self.audio_outputs.insert(index, FftAudioOutput::default());
        index
    }

    /// Adds a processor node to the graph and returns its index.
    pub fn add(&mut self, processor: impl FftProcessor) -> NodeIndex {
        self.digraph.add_node(FftProcessorNode::new(processor))
    }

    /// Connects the output of one node to the input of another node.
    ///
    /// If there is already a connection to the target input, it will be replaced.
    pub fn connect(
        &mut self,
        source: NodeIndex,
        source_output: usize,
        target: NodeIndex,
        target_input: usize,
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
            FftEdge {
                source_output,
                target_input,
            },
        );

        self.reset_visitor();
    }

    fn reset_visitor(&mut self) {
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
    }

    /// Allocates memory for the graph based on the given parameters.
    pub fn allocate(&mut self, block_size: usize) {
        self.reset_visitor();

        let fft_length = self.fft_length();

        for node_id in &self.visit_path {
            let node = self.digraph.node_weight_mut(*node_id).unwrap();
            let mut buffers = Vec::new();
            for out in node.output_spec() {
                match out.signal_type {
                    FftSignalType::RealBuf(length) => {
                        let buf =
                            vec![0.0; length.calculate(fft_length, block_size)].into_boxed_slice();
                        buffers.push(FftSignal::RealBuf(RealBuf(buf)));
                    }
                    FftSignalType::ComplexBuf(length) => {
                        let buf =
                            vec![Complex::default(); length.calculate(fft_length, block_size)]
                                .into_boxed_slice();
                        buffers.push(FftSignal::ComplexBuf(ComplexBuf(buf)));
                    }
                    FftSignalType::Param(_) => {}
                }
            }

            self.buffer_cache.insert(*node_id, buffers);

            node.allocate(self.fft_length, self.fft_length * 2);
        }

        for input in self.audio_inputs.values_mut() {
            input.ring_buffer = VecDeque::with_capacity(self.fft_length + block_size);
            input.time_domain =
                FftSignal::RealBuf(RealBuf(vec![0.0; self.fft_length * 2].into_boxed_slice()));
        }

        for output in self.audio_outputs.values_mut() {
            output.ring_buffer = VecDeque::with_capacity(self.fft_length + block_size);
            output.overlap_buffer = VecDeque::with_capacity(self.fft_length + block_size);
        }
    }

    #[cfg_attr(feature = "profiling", inline(never))]
    #[allow(clippy::needless_range_loop)]
    fn process_inner(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        let fft_length = self.fft_length();
        let hop_length = self.hop_length();

        let mut input_buffer_len = 0;
        for input_index in 0..self.inputs.len() {
            let Some(inp) = self.audio_inputs.get_mut(&self.inputs[input_index]) else {
                continue;
            };

            let input = inputs.input(input_index).unwrap();
            let input = input.as_type::<f32>().unwrap();

            // fill the input buffer
            for i in 0..input.len() {
                inp.ring_buffer.push_back(input[i].unwrap_or_default());
            }

            input_buffer_len = inp.ring_buffer.len();
        }

        // while we still have enough samples to process
        while input_buffer_len >= fft_length {
            for input_index in 0..self.inputs.len() {
                let Some(inp) = self.audio_inputs.get_mut(&self.inputs[input_index]) else {
                    continue;
                };

                let time_domain = inp.time_domain.as_real_buf_mut().unwrap();
                for i in 0..fft_length {
                    time_domain[i] = inp.ring_buffer[i] * self.window[i];
                }
                time_domain[fft_length..].fill(0.0);

                // advance the input buffer
                inp.ring_buffer.drain(..hop_length);
            }

            // run the FFT processor nodes
            for i in 0..self.visit_path.len() {
                let node_id = self.visit_path[i];
                self.process_node(node_id)?;
            }

            for output_index in 0..self.outputs.len() {
                let Some(out) = self.audio_outputs.get_mut(&self.outputs[output_index]) else {
                    continue;
                };

                let FftAudioOutput {
                    ring_buffer,
                    overlap_buffer,
                } = out;

                let buffers = self.buffer_cache.get(&self.outputs[output_index]).unwrap();

                let FftSignal::RealBuf(output_buf) = &buffers[0] else {
                    continue;
                };

                // overlap-add
                for i in 0..fft_length * 2 {
                    if i < overlap_buffer.len() {
                        overlap_buffer[i] += output_buf[i];
                    } else {
                        overlap_buffer.push_back(output_buf[i]);
                    }
                }

                ring_buffer.extend(overlap_buffer.drain(..hop_length));
            }

            // we just consumed `hop_length` samples from each input buffer
            input_buffer_len -= hop_length;
        }

        // for each output, write as much of the output's ring buffer as possible to the block's corresponding output buffer
        for output_index in 0..self.outputs.len() {
            let Some(audio_out) = self.audio_outputs.get_mut(&self.outputs[output_index]) else {
                continue;
            };

            let mut proc_out = outputs.output(output_index);

            for i in 0..inputs.block_size() {
                if let Some(sample) = audio_out.ring_buffer.pop_front() {
                    proc_out.set_as(i, sample);
                } else {
                    proc_out.set_as(i, 0.0);
                }
            }
        }

        Ok(())
    }

    #[cfg_attr(feature = "profiling", inline(never))]
    fn process_node(&mut self, node_id: NodeIndex) -> Result<(), ProcessorError> {
        let mut inputs = BTreeMap::new();
        // let mut inputs = smallvec::SmallVec::<[&FftSignal; 4]>::new();
        let mut outputs = self.buffer_cache.remove(&node_id).unwrap();

        if let Some(inp) = self.audio_inputs.get(&node_id) {
            inputs.insert(0, &inp.time_domain);
        } else {
            for (source, edge) in self
                .digraph
                .edges_directed(node_id, Direction::Incoming)
                .map(|e| (e.source(), e.weight()))
            {
                let source_buffers = self.buffer_cache.get(&source).unwrap();
                let input = &source_buffers[edge.source_output];
                inputs.insert(edge.target_input, input);
            }
        }

        let inputs: smallvec::SmallVec<[_; 4]> = inputs.into_iter().map(|(_, v)| v).collect();

        self.digraph[node_id].process(self.fft_length, &inputs, &mut outputs)?;

        drop(inputs);
        self.buffer_cache.insert(node_id, outputs);

        Ok(())
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for FftGraph {
    fn input_spec(&self) -> Vec<SignalSpec> {
        let mut specs = Vec::new();
        for i in 0..self.inputs.len() {
            specs.push(SignalSpec::new(i.to_string(), SignalType::f32));
        }
        specs
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        let mut specs = Vec::new();
        for i in 0..self.outputs.len() {
            specs.push(SignalSpec::new(i.to_string(), SignalType::f32));
        }
        specs
    }

    fn allocate(&mut self, _sample_rate: f32, max_block_size: usize) {
        self.allocate(max_block_size);
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        self.process_inner(inputs, outputs)
    }
}
