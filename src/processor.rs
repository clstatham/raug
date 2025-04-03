//! Audio processing utilities and types.

use std::fmt::Debug;

use downcast_rs::{Downcast, impl_downcast};
use itertools::Either;
use thiserror::Error;

use crate::{
    GraphSerde,
    graph::asset::{AssetRef, Assets},
    signal::{
        AnySignal, AnySignalOpt, AnySignalOptMut, Float, OptRepr, OptSignal, Signal, SignalBuffer,
        SignalType,
    },
};

/// Error type for [`Processor`] operations.
#[derive(Debug, Clone, Error)]
pub enum ProcessorError {
    /// The number of inputs must match the number returned by [`Processor::num_inputs()`].
    #[error("The number of inputs must match the number returned by Processor::num_inputs()")]
    NumInputsMismatch,

    /// The number of outputs must match the number returned by [`Processor::num_outputs()`].
    #[error("The number of outputs must match the number returned by Processor::num_outputs()")]
    NumOutputsMismatch,

    /// Input signal type mismatch.
    #[error("Input {index} signal type mismatch (expected {expected:?}, got {actual:?})")]
    InputSpecMismatch {
        /// The index of the input signal.
        index: usize,
        /// The expected signal type.
        expected: SignalType,
        /// The actual signal type.
        actual: SignalType,
    },

    /// Output signal type mismatch.
    #[error("Output {index} signal type mismatch (expected {expected:?}, got {actual:?})")]
    OutputSpecMismatch {
        /// The index of the output signal.
        index: usize,
        /// The expected signal type.
        expected: SignalType,
        /// The actual signal type.
        actual: SignalType,
    },

    /// Invalid value.
    #[error("Invalid value: {0}")]
    InvalidValue(&'static str),

    /// Invalid cast.
    #[error("Invalid cast: {0:?} to {1:?}")]
    InvalidCast(SignalType, SignalType),

    #[error("Sub-graph error: {0}")]
    SubGraph(#[from] Box<crate::graph::GraphRunError>),

    #[error("Asset `{0}` type invalid: {0}")]
    InvalidAsset(String, String),

    #[error("Asset `{0}` not found")]
    AssetNotFound(String),

    #[cfg(feature = "fft")]
    /// FFT error.
    #[error("FFT error: {0}")]
    Fft(#[from] crate::fft::FftError),

    #[error("Other error")]
    Other,
}

/// Information about an input or output of a [`Processor`].
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SignalSpec {
    /// The name of the input or output.
    pub name: String,
    /// The type of the input or output.
    pub signal_type: SignalType,
}

impl Default for SignalSpec {
    fn default() -> Self {
        Self {
            name: "".into(),
            signal_type: SignalType::Float,
        }
    }
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
pub struct ProcEnv<'a> {
    pub assets: &'a Assets,
    pub sample_rate: Float,
    pub block_size: usize,
    pub mode: ProcessMode,
}

impl ProcEnv<'_> {
    pub fn asset(&self, name: &str) -> Option<AssetRef> {
        self.assets.get(name)
    }
}

/// A collection of input signals for a [`Processor`] and their specifications.
#[derive(Clone, Copy)]
pub struct ProcessorInputs<'a> {
    /// The specifications of the input signals.
    pub input_specs: &'a [SignalSpec],

    /// The input signals.
    pub inputs: &'a [Option<*const SignalBuffer>],

    /// Environment information for the processor.
    pub env: ProcEnv<'a>,
}

impl<'a> ProcessorInputs<'a> {
    /// Creates a new collection of input signals.
    #[inline]
    pub fn new(
        input_specs: &'a [SignalSpec],
        inputs: &'a [Option<*const SignalBuffer>],
        env: ProcEnv<'a>,
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
    pub fn sample_rate(&self) -> Float {
        self.env.sample_rate
    }

    /// Returns the current block size.
    #[inline]
    pub fn block_size(&self) -> usize {
        self.env.block_size
    }

    /// Returns the asset with the given name, if it exists.
    #[inline]
    pub fn asset(&self, name: &str) -> Result<AssetRef, ProcessorError> {
        self.env
            .assets
            .get(name)
            .ok_or_else(|| ProcessorError::AssetNotFound(name.into()))
    }

    /// Returns the input signal at the given index. Unconnected inputs are represented as `None`.
    #[inline]
    pub fn input(&self, index: usize) -> Option<&SignalBuffer> {
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
    pub fn input_as<S: Signal>(&self, index: usize) -> Option<&[Option<S::Repr>]> {
        let input = self.input(index)?;
        Some(input.as_type::<S>()?.as_ref())
    }

    /// Returns an iterator over the input signal at the given index.
    #[inline]
    pub fn iter_input(&self, index: usize) -> impl Iterator<Item = Option<AnySignalOpt>> + use<'_> {
        if let Some(buffer) = self.input(index) {
            if let ProcessMode::Sample(sample_index) = self.env.mode {
                Ternary::B(std::iter::once(buffer.get(sample_index)))
            } else {
                Ternary::A(buffer.iter().map(Some))
            }
        } else {
            Ternary::C(std::iter::repeat(None))
        }
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
            if buffer.signal_type().is_same_as(&S::signal_type()) {
                Ok(Ternary::B(std::iter::once(
                    buffer.as_type::<S>().unwrap()[sample_index].map(S::from_repr),
                )))
            } else {
                Err(ProcessorError::InputSpecMismatch {
                    index,
                    expected: S::signal_type(),
                    actual: buffer.signal_type(),
                })
            }
        } else if buffer.signal_type().is_same_as(&S::signal_type()) {
            Ok(Ternary::A(
                buffer
                    .as_type::<S>()
                    .unwrap()
                    .iter()
                    .copied()
                    .map(|s| s.map(S::from_repr)),
            ))
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
#[derive(Debug)]
pub enum ProcessorOutput<'a> {
    /// A block of signals.
    Block(&'a mut SignalBuffer),
    /// A single sample.
    Sample(&'a mut SignalBuffer, usize),
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

    /// Returns an iterator over the output signal.
    #[inline]
    pub fn iter_mut(&'a mut self) -> impl Iterator<Item = AnySignalOptMut<'a>> {
        match self {
            ProcessorOutput::Block(buffer) => Either::Left(buffer.iter_mut()),
            ProcessorOutput::Sample(buffer, sample_index) => {
                Either::Right(std::iter::once(buffer.get_mut(*sample_index).unwrap()))
            }
        }
    }

    /// Returns an iterator over the output signal, if it is of the given type.
    #[inline]
    pub fn iter_mut_as<S: Signal>(&'a mut self) -> impl Iterator<Item = &'a mut Option<S::Repr>> {
        match self {
            ProcessorOutput::Block(buffer) => {
                Either::Left(buffer.as_type_mut::<S>().unwrap().iter_mut())
            }
            ProcessorOutput::Sample(buffer, sample_index) => Either::Right(std::iter::once(
                &mut buffer.as_type_mut::<S>().unwrap()[*sample_index],
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
            ProcessorOutput::Block(buffer) => {
                buffer.as_type::<S>().unwrap().get(index)?.into_signal()
            }
            ProcessorOutput::Sample(buffer, sample_index) => buffer
                .as_type::<S>()
                .unwrap()
                .get(*sample_index)?
                .into_signal(),
        }
    }

    /// Sets the output signal at the given index.
    ///
    /// # Panics
    ///
    /// Panics if the output signal is not the same type as the given value.
    #[inline]
    pub fn set(&mut self, index: usize, value: impl Into<AnySignalOpt>) {
        match self {
            ProcessorOutput::Block(buffer) => {
                let value: AnySignalOpt = value.into();
                buffer.set(index, value);
            }
            ProcessorOutput::Sample(buffer, sample_index) => {
                let value: AnySignalOpt = value.into();
                buffer.set(*sample_index, value)
            }
        }
    }

    /// Sets the output signal at the given index, if it is of the given type.
    ///
    /// # Panics
    ///
    /// Panics if the output signal is not of the given type.
    #[inline]
    pub fn set_as<S: Signal>(&mut self, index: usize, value: impl Into<Option<S>>) {
        match self {
            ProcessorOutput::Block(buffer) => {
                buffer.as_type_mut::<S>().unwrap()[index] = value.into().into_repr();
            }
            ProcessorOutput::Sample(buffer, sample_index) => {
                buffer.as_type_mut::<S>().unwrap()[*sample_index] = value.into().into_repr();
            }
        }
    }

    /// Sets the output signal at the given index to `None`.
    #[inline]
    pub fn set_none(&mut self, index: usize) {
        match self {
            ProcessorOutput::Block(buffer) => buffer.set_none(index),
            ProcessorOutput::Sample(buffer, sample_index) => buffer.set_none(*sample_index),
        }
    }

    /// Fills the output signal with the given value, if it is of the correct type.
    #[inline]
    pub fn fill_as<S: Signal + Clone>(&mut self, value: impl Into<Option<S>>) {
        match self {
            ProcessorOutput::Block(buffer) => buffer
                .as_type_mut::<S>()
                .unwrap()
                .fill(value.into().into_repr()),
            ProcessorOutput::Sample(buffer, sample_index) => {
                buffer.as_type_mut::<S>().unwrap()[*sample_index] = value.into().into_repr();
            }
        }
    }

    /// Fills the output signal with the given value.
    #[inline]
    pub fn fill(&mut self, value: AnySignalOpt) {
        match self {
            ProcessorOutput::Block(buffer) => buffer.fill(value),
            ProcessorOutput::Sample(buffer, sample_index) => {
                buffer.set(*sample_index, value);
            }
        }
    }

    /// Fills the output signal with `None`.
    #[inline]
    pub fn fill_none(&mut self) {
        match self {
            ProcessorOutput::Block(buffer) => buffer.fill_none(),
            ProcessorOutput::Sample(buffer, sample_index) => {
                buffer.set_none(*sample_index);
            }
        }
    }
}

/// A collection of output signals for a [`Processor`] and their specifications.
pub struct ProcessorOutputs<'a> {
    /// The specifications of the output signals.
    pub output_spec: &'a [SignalSpec],

    /// The output signals.
    pub outputs: &'a mut [SignalBuffer],

    /// The mode in which the processor should process signals.
    pub mode: ProcessMode,
}

impl<'a> ProcessorOutputs<'a> {
    #[inline]
    /// Creates a new collection of output signals.
    pub fn new(
        output_spec: &'a [SignalSpec],
        outputs: &'a mut [SignalBuffer],
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
    pub fn set_output_none(&mut self, output_index: usize, sample_index: usize) {
        self.outputs[output_index].set_none(sample_index);
    }

    /// Sets the output signal at the given index.
    #[inline]
    pub fn set_output(
        &mut self,
        output_index: usize,
        sample_index: usize,
        signal: AnySignal,
    ) -> Result<(), ProcessorError> {
        if signal.signal_type() != self.output_spec[output_index].signal_type {
            return Err(ProcessorError::OutputSpecMismatch {
                index: output_index,
                expected: self.output_spec[output_index].signal_type,
                actual: signal.signal_type(),
            });
        }

        self.outputs[output_index].set(sample_index, signal.into_any_signal_opt());

        Ok(())
    }

    /// Sets the output signal at the given index.
    #[inline]
    pub fn set_output_as<S: Signal>(
        &mut self,
        output_index: usize,
        sample_index: usize,
        signal: S,
    ) -> Result<(), ProcessorError> {
        let signal = signal.into_any_signal();

        if signal.signal_type() != self.output_spec[output_index].signal_type {
            return Err(ProcessorError::OutputSpecMismatch {
                index: output_index,
                expected: self.output_spec[output_index].signal_type,
                actual: signal.signal_type(),
            });
        }

        self.outputs[output_index].set(sample_index, signal.into_any_signal_opt());

        Ok(())
    }

    /// Returns an iterator over the output signal at the given index.
    #[inline]
    pub fn iter_output_mut(&mut self, index: usize) -> impl Iterator<Item = AnySignalOptMut> {
        let output = &mut self.outputs[index];
        if let ProcessMode::Sample(sample_index) = self.mode {
            Either::Left(std::iter::once(output.get_mut(sample_index).unwrap()))
        } else {
            Either::Right(output.iter_mut())
        }
    }

    /// Returns an iterator over the output signal at the given index, if it is of the given type.
    #[inline]
    pub fn iter_output_mut_as<S: Signal>(
        &mut self,
        index: usize,
    ) -> Result<impl Iterator<Item = &mut Option<S::Repr>> + '_, ProcessorError> {
        if let ProcessMode::Sample(sample_index) = self.mode {
            let output = &mut self.outputs[index];
            if output.signal_type().is_same_as(&S::signal_type()) {
                Ok(Either::Left(std::iter::once(
                    &mut output.as_type_mut::<S>().unwrap()[sample_index],
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
            let actual = output.signal_type();
            let output =
                output
                    .as_type_mut::<S>()
                    .ok_or_else(|| ProcessorError::OutputSpecMismatch {
                        index,
                        expected: S::signal_type(),
                        actual,
                    })?;

            Ok(Either::Right(output.iter_mut()))
        }
    }
}

/// A processor that can process audio signals.
#[cfg_attr(feature = "serde", typetag::serde(tag = "type"))]
pub trait Processor
where
    Self: Downcast + ProcessorClone + GraphSerde + Send,
{
    /// Returns the name of the processor.
    fn name(&self) -> &str {
        let type_name = std::any::type_name::<Self>();
        let has_generics = type_name.contains('<');
        if has_generics {
            let end = type_name.find('<').unwrap();
            let start = type_name[..end].rfind(':').map_or(0, |i| i + 1);
            &type_name[start..end]
        } else {
            type_name.rsplit(':').next().unwrap()
        }
    }

    /// Returns the specifications of the input signals of the processor.
    fn input_spec(&self) -> Vec<SignalSpec>;

    /// Returns the specifications of the output signals of the processor.
    fn output_spec(&self) -> Vec<SignalSpec>;

    /// Returns the number of input signals required by the processor.
    fn num_inputs(&self) -> usize {
        self.input_spec().len()
    }

    /// Returns the number of output signals produced by the processor.
    fn num_outputs(&self) -> usize {
        self.output_spec().len()
    }

    /// Called once, before processing starts.
    ///
    /// Do all of your preallocation here.
    #[allow(unused)]
    fn allocate(&mut self, sample_rate: Float, max_block_size: usize) {}

    /// Called anytime the sample rate or block size changes.
    ///
    /// This function is NOT ALLOWED to allocate memory.
    #[allow(unused)]
    fn resize_buffers(&mut self, sample_rate: Float, block_size: usize) {}

    /// Processes the input signals and writes the output signals.
    ///
    /// This function is NOT ALLOWED to allocate memory.
    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError>;
}
impl_downcast!(Processor);

mod sealed {
    pub trait Sealed {}
    impl<T: Clone> Sealed for T {}
}

#[doc(hidden)]
pub trait ProcessorClone: sealed::Sealed {
    fn clone_boxed(&self) -> Box<dyn Processor>;
}

impl<T> ProcessorClone for T
where
    T: Clone + Processor,
{
    fn clone_boxed(&self) -> Box<dyn Processor> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Processor> {
    fn clone(&self) -> Self {
        self.clone_boxed()
    }
}

impl Debug for dyn Processor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}
