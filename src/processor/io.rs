use itertools::Either;

use crate::signal::{Signal, SignalType, type_erased::ErasedBuffer};

use super::ProcessorError;

/// Information about an input or output of a [`Processor`].
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
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Ternary<A, B, C> {
    A(A),
    B(B),
    C(C),
}

impl<A, B, C> Iterator for Ternary<A, B, C>
where
    A: Iterator,
    B: Iterator<Item = A::Item>,
    C: Iterator<Item = A::Item>,
{
    type Item = A::Item;

    #[inline] // this function is ***VERY*** hot - do not change this
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Ternary::A(a) => a.next(),
            Ternary::B(b) => b.next(),
            Ternary::C(c) => c.next(),
        }
    }
}

/// The mode in which a processor should process signals.
///
/// - `Block` means the processor processes the entire block of samples at once.
/// - `Sample` means the processor processes each sample individually.
#[derive(Debug, Clone, Copy)]
pub enum ProcessMode {
    /// The processor should process the entire block of samples at once.
    Block,
    /// The processor should process the sample at the given index.
    Sample(
        /// The index of the current sample within the block.
        usize,
    ),
}

#[derive(Debug, Clone, Copy)]
pub struct ProcEnv {
    pub sample_rate: f32,
    pub block_size: usize,
    pub mode: ProcessMode,
}

/// A collection of input signals for a [`Processor`] and their specifications.
#[derive(Clone, Copy)]
pub struct ProcessorInputs<'a> {
    /// The specifications of the input signals.
    pub input_specs: &'a [SignalSpec],

    /// The input signals.
    pub inputs: &'a [Option<*const ErasedBuffer>],

    /// Environment information for the processor.
    pub env: ProcEnv,
}

impl<'a> ProcessorInputs<'a> {
    /// Creates a new collection of input signals.
    #[inline]
    pub fn new(
        input_specs: &'a [SignalSpec],
        inputs: &'a [Option<*const ErasedBuffer>],
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

    /// Returns the input signal at the given index. Unconnected inputs are represented as `None`.
    #[inline]
    pub fn input(&self, index: usize) -> Option<&ErasedBuffer> {
        let ptr = self
            .inputs
            .get(index)
            .and_then(|input| input.as_ref().copied())?;
        // SAFETY: The pointer is valid because ProcessorInputs is only created
        // during `Runtime::process_node` which limits the lifetime of the inputs to the
        // lifetime of that call.
        let buffer = unsafe { &*ptr };
        Some(buffer)
    }

    #[inline]
    pub fn input_as<S: Signal>(&self, index: usize) -> Option<&[S]> {
        let input = self.input(index)?;
        Some(input.as_slice::<S>())
    }

    /// Returns an iterator over the input signal at the given index, if it is of the given type.
    #[inline]
    pub fn iter_input_as<S: Signal>(
        &self,
        index: usize,
    ) -> Result<impl Iterator<Item = Option<S>> + '_, ProcessorError> {
        let Some(buffer) = self.input(index) else {
            return Ok(Ternary::C(std::iter::repeat(None)));
        };

        if let ProcessMode::Sample(sample_index) = self.env.mode {
            if buffer.signal_type() == S::signal_type() {
                Ok(Ternary::B(std::iter::once(Some(
                    buffer.as_slice::<S>()[sample_index],
                ))))
            } else {
                Err(ProcessorError::InputSpecMismatch {
                    index,
                    expected: S::signal_type(),
                    actual: buffer.signal_type(),
                })
            }
        } else if buffer.signal_type() == S::signal_type() {
            Ok(Ternary::A(buffer.as_slice::<S>().iter().copied().map(Some)))
        } else {
            Err(ProcessorError::InputSpecMismatch {
                index,
                expected: S::signal_type(),
                actual: buffer.signal_type(),
            })
        }
    }
}

/// The output of a [`Processor`].
pub enum ProcessorOutput<'a> {
    /// A block of signals.
    Block(&'a mut ErasedBuffer),
    /// A single sample.
    Sample(&'a mut ErasedBuffer, usize),
}

impl<'a> ProcessorOutput<'a> {
    /// Returns the type of the output signal.
    #[inline]
    pub fn signal_type(&self) -> SignalType {
        match self {
            ProcessorOutput::Block(buffer) => buffer.signal_type(),
            ProcessorOutput::Sample(buffer, _) => buffer.signal_type(),
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

    /// Returns an iterator over the output signal, if it is of the given type.
    #[inline]
    pub fn iter_mut_as<S: Signal>(&'a mut self) -> impl Iterator<Item = &'a mut S> {
        match self {
            ProcessorOutput::Block(buffer) => Either::Left(buffer.as_mut_slice::<S>().iter_mut()),
            ProcessorOutput::Sample(buffer, sample_index) => Either::Right(std::iter::once(
                &mut buffer.as_mut_slice::<S>()[*sample_index],
            )),
        }
    }

    /// Returns a reference to the output signal at the given index, if it is of the given type.
    ///
    /// # Panics
    ///
    /// Panics if the output signal is not of the given type.
    #[inline]
    pub fn get_as<S: Signal>(&self, index: usize) -> Option<S> {
        match self {
            ProcessorOutput::Block(buffer) => buffer.as_slice::<S>().get(index).copied(),
            ProcessorOutput::Sample(buffer, sample_index) => {
                buffer.as_slice::<S>().get(*sample_index).copied()
            }
        }
    }

    /// Sets the output signal at the given index, if it is of the given type.
    ///
    /// If the output is in per-sample mode, the index is ignored and the signal is set at the current sample index.
    ///
    /// # Panics
    ///
    /// Panics if the output signal is not of the given type.
    #[inline]
    pub fn set_as<S: Signal>(&mut self, index: usize, value: impl Into<S>) {
        match self {
            ProcessorOutput::Block(buffer) => {
                buffer.set_as::<S>(index, value.into());
            }
            ProcessorOutput::Sample(buffer, sample_index) => {
                buffer.set_as::<S>(*sample_index, value.into());
            }
        }
    }

    /// Sets the output signal at the given index to `None`.
    #[inline]
    pub fn set_default<S: Signal>(&mut self, index: usize) {
        match self {
            ProcessorOutput::Block(buffer) => {
                buffer.set_as::<S>(index, S::default());
            }
            ProcessorOutput::Sample(buffer, sample_index) => {
                buffer.set_as::<S>(*sample_index, S::default());
            }
        }
    }

    /// Fills the output signal with the given value, if it is of the correct type.
    #[inline]
    pub fn fill_as<S: Signal + Clone>(&mut self, value: impl Into<S>) {
        match self {
            ProcessorOutput::Block(buffer) => buffer.as_mut_slice::<S>().fill(value.into()),
            ProcessorOutput::Sample(buffer, sample_index) => {
                buffer.as_mut_slice::<S>()[*sample_index] = value.into();
            }
        }
    }
}

/// A collection of output signals for a [`Processor`] and their specifications.
pub struct ProcessorOutputs<'a> {
    /// The specifications of the output signals.
    pub output_spec: &'a [SignalSpec],

    /// The output signals.
    pub outputs: &'a mut [ErasedBuffer],

    /// The mode in which the processor should process signals.
    pub mode: ProcessMode,
}

impl<'a> ProcessorOutputs<'a> {
    #[inline]
    /// Creates a new collection of output signals.
    pub fn new(
        output_spec: &'a [SignalSpec],
        outputs: &'a mut [ErasedBuffer],
        mode: ProcessMode,
    ) -> Self {
        Self {
            output_spec,
            outputs,
            mode,
        }
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

    /// Returns the specification of the output signal at the given index.
    #[inline]
    pub fn output_spec(&self, index: usize) -> &SignalSpec {
        &self.output_spec[index]
    }

    /// Sets the output signal at the given index to `None`.
    #[inline]
    pub fn set_output_default<S: Signal>(&mut self, output_index: usize, sample_index: usize) {
        self.outputs[output_index].set_as::<S>(sample_index, S::default());
    }

    /// Sets the output signal at the given index.
    #[inline]
    pub fn set_output_as<S: Signal>(
        &mut self,
        output_index: usize,
        sample_index: usize,
        signal: S,
    ) -> Result<(), ProcessorError> {
        if S::signal_type() != self.output_spec[output_index].signal_type {
            return Err(ProcessorError::OutputSpecMismatch {
                index: output_index,
                expected: self.output_spec[output_index].signal_type,
                actual: S::signal_type(),
            });
        }

        self.outputs[output_index].set_as::<S>(sample_index, signal);

        Ok(())
    }

    /// Returns an iterator over the output signal at the given index, if it is of the given type.
    #[inline]
    pub fn iter_output_mut_as<S: Signal>(
        &mut self,
        index: usize,
    ) -> Result<impl Iterator<Item = &mut S> + '_, ProcessorError> {
        if let ProcessMode::Sample(sample_index) = self.mode {
            let output = &mut self.outputs[index];
            if output.signal_type() == S::signal_type() {
                Ok(Either::Left(std::iter::once(
                    &mut output.as_mut_slice::<S>()[sample_index],
                )))
            } else {
                Err(ProcessorError::OutputSpecMismatch {
                    index,
                    expected: S::signal_type(),
                    actual: output.signal_type(),
                })
            }
        } else {
            let output = &mut self.outputs[index];
            let output = output.as_mut_slice::<S>();

            Ok(Either::Right(output.iter_mut()))
        }
    }
}
