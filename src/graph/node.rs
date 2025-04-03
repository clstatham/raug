//! Contains the [`ProcessorNode`] struct, which represents a node in the audio graph that processes signals.

use std::fmt::Debug;

use crate::{
    prelude::{ProcEnv, Processor, ProcessorError, ProcessorInputs, ProcessorOutputs, SignalSpec},
    processor::ProcessMode,
    signal::{Float, SignalBuffer},
};

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
            outputs.push(SignalBuffer::new_of_type(&spec.signal_type, 0));
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
    pub fn allocate(&mut self, sample_rate: Float, max_block_size: usize) {
        self.processor.allocate(sample_rate, max_block_size);
        for (spec, output) in self
            .output_spec
            .iter()
            .zip(self.outputs.as_mut().unwrap().iter_mut())
        {
            output.resize_with_hint(max_block_size, &spec.signal_type);
        }
    }

    /// Resizes the internal buffers of the processor and updates the sample rate and block size.
    ///
    /// This function is NOT ALLOWED to allocate memory.
    #[inline]
    pub fn resize_buffers(&mut self, sample_rate: Float, block_size: usize) {
        self.processor.resize_buffers(sample_rate, block_size);
    }

    /// Processes the input signals and writes the output signals to the given buffers.
    #[inline]
    pub(crate) fn process(
        &mut self,
        inputs: &[Option<*const SignalBuffer>],
        env: ProcEnv<'_>,
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
