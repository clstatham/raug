//! Contains the [`ProcessorNode`] and [`Node`] structs, which represent nodes in the audio graph that process signals.

use std::{fmt::Debug, sync::Arc};

use parking_lot::Mutex;
use raug_graph::node::{Node as AbstractNode, NodeIndexExt};
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

pub trait AsNodeOutput<I: AsNodeOutputIndex<ProcessorNode>> {
    fn as_node_output(&self, graph: &mut Graph) -> NodeOutput<ProcessorNode, I>;
}

impl<O: AsNodeOutputIndex<ProcessorNode>> AsNodeOutput<O> for NodeOutput<ProcessorNode, O> {
    fn as_node_output(&self, _graph: &mut Graph) -> NodeOutput<ProcessorNode, O> {
        *self
    }
}

impl<S: Signal + Default + Clone> AsNodeOutput<u32> for S {
    fn as_node_output(&self, graph: &mut Graph) -> NodeOutput<ProcessorNode, u32> {
        let node = graph.constant(self.clone());
        node.output(0)
    }
}

impl AsNodeOutput<u32> for NodeIndex {
    fn as_node_output(&self, _graph: &mut Graph) -> NodeOutput<ProcessorNode, u32> {
        NodeOutput::new(*self, 0)
    }
}
