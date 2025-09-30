//! Input and output signal handling for processors.

use crate::{
    prelude::AnySignalRef,
    signal::{Signal, SignalType, type_erased::AnyBuffer},
};

use super::ProcessorError;

/// Information about an input or output of a [`Processor`](super::Processor).
#[derive(Debug, Clone)]
pub struct SignalSpec {
    /// The name of the input or output.
    pub name: String,
    /// The type of the input or output.
    pub signal_type: SignalType,
}

impl SignalSpec {
    /// Creates a new [`SignalSpec`] with the given name and type.
    pub fn new(name: impl Into<String>, signal_type: SignalType) -> Self {
        Self {
            name: name.into(),
            signal_type,
        }
    }

    /// Creates a new [`SignalSpec`] with the given name and type.
    pub fn of_type<S: Signal>(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            signal_type: S::signal_type(),
        }
    }
}

/// The mode in which a processor is processsing signals.
///
/// - `Block` means the processor processes the entire block of samples at once.
/// - `Sample` means the processor processes each sample individually.
#[derive(Debug, Clone, Copy)]
pub enum ProcessMode {
    /// The processor is processing an entire block of samples.
    Block,
    /// The processor is processing a single sample within a block.
    Sample(
        /// The index of the current sample within the block.
        usize,
    ),
}

/// Environment information for a [`Processor`](super::Processor).
#[derive(Debug, Clone, Copy)]
pub struct ProcEnv {
    /// The sample rate of the audio engine.
    pub sample_rate: f32,
    /// The block size of the audio engine.
    pub block_size: usize,
    /// The mode in which the processor is processing signals.
    pub mode: ProcessMode,
}

/// A collection of input signals for a [`Processor`](super::Processor) and their specifications.
#[derive(Clone, Copy)]
pub struct ProcessorInputs<'a> {
    /// The specifications of the input signals.
    pub input_specs: &'a [SignalSpec],

    /// The input signals.
    pub inputs: &'a [Option<*const AnyBuffer>],

    /// Environment information for the processor.
    pub env: ProcEnv,
}

impl<'a> ProcessorInputs<'a> {
    /// Creates a new collection of input signals.
    #[inline]
    pub fn new(
        input_specs: &'a [SignalSpec],
        inputs: &'a [Option<*const AnyBuffer>],
        env: ProcEnv,
    ) -> Self {
        Self {
            input_specs,
            inputs,
            env,
        }
    }

    /// Returns the number of input signals.
    #[inline]
    pub fn num_inputs(&self) -> usize {
        self.input_specs.len()
    }

    /// Returns the specification of the input signal at the given index.
    #[inline]
    pub fn input_spec(&self, index: usize) -> &SignalSpec {
        &self.input_specs[index]
    }

    /// Returns the current sample rate.
    #[inline]
    pub fn sample_rate(&self) -> f32 {
        self.env.sample_rate
    }

    /// Returns the current block size.
    #[inline]
    pub fn block_size(&self) -> usize {
        self.env.block_size
    }

    /// Returns the input signal at the given index.
    /// Unconnected inputs are represented as `None`.
    #[inline]
    pub fn input(&self, index: usize) -> Option<&AnyBuffer> {
        let ptr = self
            .inputs
            .get(index)
            .and_then(|input| input.as_ref().copied())?;
        // SAFETY: The pointer is valid because ProcessorInputs is only created
        // during `Graph::process_node` which limits the lifetime of the inputs to the
        // lifetime of that call.
        let buffer = unsafe { &*ptr };
        Some(buffer)
    }

    /// Returns the input signal at the given index, if it is of the given type.
    /// Unconnected inputs are represented as `None`.
    #[inline]
    pub fn input_as<S: Signal>(&self, index: usize) -> Option<&[S]> {
        let input = self.input(index)?;
        input.as_slice::<S>()
    }
}

/// The output of a [`Processor`](super::Processor).
pub enum ProcessorOutput<'a> {
    /// A block of signals.
    Block(&'a mut AnyBuffer),
    /// A single sample.
    Sample(&'a mut AnyBuffer, usize),
}

impl ProcessorOutput<'_> {
    /// Returns the type of the output signal.
    #[inline]
    pub fn signal_type(&self) -> SignalType {
        match self {
            ProcessorOutput::Block(buffer) => buffer.element_signal_type(),
            ProcessorOutput::Sample(buffer, _) => buffer.element_signal_type(),
        }
    }

    /// Returns the number of signals in the output.
    #[inline]
    pub fn len(&self) -> usize {
        match self {
            ProcessorOutput::Block(buffer) => buffer.len(),
            ProcessorOutput::Sample(buffer, _) => buffer.len(),
        }
    }

    /// Returns `true` if the output signal is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        match self {
            ProcessorOutput::Block(buffer) => buffer.is_empty(),
            ProcessorOutput::Sample(buffer, _) => buffer.is_empty(),
        }
    }

    /// Returns a reference to the output signal at the given index, if it is of the given type.
    #[inline]
    pub fn get_as<S: Signal>(&self, index: usize) -> Option<&S> {
        match self {
            ProcessorOutput::Block(buffer) => buffer.as_slice::<S>()?.get(index),
            ProcessorOutput::Sample(buffer, sample_index) => {
                buffer.as_slice::<S>()?.get(*sample_index)
            }
        }
    }

    /// Returns a mutable reference to the output signal at the given index, if it is of the given type.
    #[inline]
    pub fn get_mut_as<S: Signal>(&mut self, index: usize) -> Option<&mut S> {
        match self {
            ProcessorOutput::Block(buffer) => buffer.as_mut_slice::<S>()?.get_mut(index),
            ProcessorOutput::Sample(buffer, sample_index) => {
                buffer.as_mut_slice::<S>()?.get_mut(*sample_index)
            }
        }
    }

    #[inline]
    pub fn clone_from(&mut self, buf: &AnyBuffer) {
        match self {
            ProcessorOutput::Block(buffer) => buffer.clone_from(buf),
            ProcessorOutput::Sample(buffer, sample_index) => {
                let mut a = buffer.get_mut(*sample_index).unwrap();
                let b = buf.get(*sample_index).unwrap();
                a.clone_from(&b);
            }
        }
    }
}

/// A collection of output signals for a [`Processor`](super::Processor) and their specifications.
pub struct ProcessorOutputs<'a> {
    /// The specifications of the output signals.
    pub output_spec: &'a [SignalSpec],

    /// The output signals.
    pub outputs: &'a mut [AnyBuffer],

    /// The mode in which the processor should process signals.
    pub mode: ProcessMode,
}

impl<'a> ProcessorOutputs<'a> {
    #[inline]
    /// Creates a new collection of output signals.
    pub fn new(
        output_spec: &'a [SignalSpec],
        outputs: &'a mut [AnyBuffer],
        mode: ProcessMode,
    ) -> Self {
        Self {
            output_spec,
            outputs,
            mode,
        }
    }

    #[inline]
    pub fn num_outputs(&self) -> usize {
        self.output_spec.len()
    }

    /// Returns the output signal at the given index.
    #[inline]
    pub fn output(&mut self, index: usize) -> ProcessorOutput<'_> {
        if let ProcessMode::Sample(sample_index) = self.mode {
            ProcessorOutput::Sample(&mut self.outputs[index], sample_index)
        } else {
            ProcessorOutput::Block(&mut self.outputs[index])
        }
    }

    /// Returns the output signal at the given index, with an extended lifetime.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it can mutably alias the output signal at the same index more than once.
    /// Must NOT be called while the a previous `ProcessorOutput` for a particular index is still in use.
    #[inline]
    pub unsafe fn output_extended_lifetime(&mut self, index: usize) -> ProcessorOutput<'a> {
        unsafe {
            if let ProcessMode::Sample(sample_index) = self.mode {
                ProcessorOutput::Sample(
                    &mut *(&mut self.outputs[index] as *mut AnyBuffer),
                    sample_index,
                )
            } else {
                ProcessorOutput::Block(&mut *(&mut self.outputs[index] as *mut AnyBuffer))
            }
        }
    }

    /// Returns the specification of the output signal at the given index.
    #[inline]
    pub fn output_spec(&self, index: usize) -> &SignalSpec {
        &self.output_spec[index]
    }

    /// Sets the output signal at the given index.
    #[inline]
    pub fn set_output_as<S: Signal + Clone>(
        &mut self,
        output_index: usize,
        sample_index: usize,
        signal: &S,
    ) -> Result<(), ProcessorError> {
        if S::signal_type() != self.output_spec[output_index].signal_type {
            return Err(ProcessorError::OutputTypeMismatch {
                index: output_index,
                expected: self.output_spec[output_index].signal_type,
                actual: S::signal_type(),
            });
        }

        self.outputs[output_index]
            .get_mut_as::<S>(sample_index)
            .unwrap()
            .clone_from(signal);

        Ok(())
    }

    #[inline]
    pub fn set_output(
        &mut self,
        output_index: usize,
        sample_index: usize,
        signal: &AnySignalRef,
    ) -> Result<(), ProcessorError> {
        if signal.signal_type() != self.output_spec[output_index].signal_type {
            return Err(ProcessorError::OutputTypeMismatch {
                index: output_index,
                expected: self.output_spec[output_index].signal_type,
                actual: signal.signal_type(),
            });
        }

        self.outputs[output_index]
            .get_mut(sample_index)
            .unwrap()
            .clone_from(signal);

        Ok(())
    }
}
