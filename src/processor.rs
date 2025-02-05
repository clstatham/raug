//! Audio processing utilities and types.

use std::fmt::Debug;

use downcast_rs::{impl_downcast, Downcast};
use thiserror::Error;

use crate::{
    graph::asset::{AssetRef, Assets},
    signal::{
        AnySignal, AnySignalOpt, AnySignalOptMut, AnySignalOptRef, Float, Signal, SignalTuple,
        SignalType,
    },
    GraphSerde,
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
pub struct ProcessorInputs<'a, 'b> {
    /// The specifications of the input signals.
    pub input_specs: &'a [SignalSpec],

    /// The input signals.
    pub inputs: &'a [AnySignalOptRef<'b>],

    /// Environment information for the processor.
    pub env: ProcEnv<'a>,
}

impl<'a, 'b> ProcessorInputs<'a, 'b> {
    /// Creates a new collection of input signals.
    #[inline]
    pub fn new(
        input_specs: &'a [SignalSpec],
        inputs: &'a [AnySignalOptRef<'b>],
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
    pub fn input(&self, index: usize) -> Result<AnySignalOptRef, ProcessorError> {
        self.inputs
            .get(index)
            .copied()
            .ok_or(ProcessorError::NumInputsMismatch)
    }

    #[inline]
    pub fn input_as<S: Signal>(&self, index: usize) -> Result<Option<S>, ProcessorError> {
        let input = self
            .inputs
            .get(index)
            .ok_or(ProcessorError::NumInputsMismatch)?;

        if let Some(input) = input.as_any_signal_ref() {
            if input.signal_type() == S::signal_type() {
                Ok(Some(*input.as_type::<S>().unwrap()))
            } else {
                Err(ProcessorError::InputSpecMismatch {
                    index,
                    expected: S::signal_type(),
                    actual: input.signal_type(),
                })
            }
        } else {
            Ok(None)
        }
    }

    #[inline]
    pub fn as_tuple<T: SignalTuple>(&self) -> Result<T::Options, ProcessorError> {
        T::from_inputs(self.inputs).ok_or(ProcessorError::NumInputsMismatch)
    }
}

/// A collection of output signals for a [`Processor`] and their specifications.
pub struct ProcessorOutputs<'a> {
    /// The specifications of the output signals.
    pub output_spec: &'a [SignalSpec],

    /// The output signals.
    pub outputs: &'a mut [AnySignalOpt],

    /// The mode in which the processor should process signals.
    pub mode: ProcessMode,
}

impl<'a> ProcessorOutputs<'a> {
    #[inline]
    /// Creates a new collection of output signals.
    pub fn new(
        output_spec: &'a [SignalSpec],
        outputs: &'a mut [AnySignalOpt],
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
    pub fn output(&mut self, index: usize) -> AnySignalOptMut<'_> {
        self.outputs[index].as_mut()
    }

    /// Returns the specification of the output signal at the given index.
    #[inline]
    pub fn output_spec(&self, index: usize) -> &SignalSpec {
        &self.output_spec[index]
    }

    /// Sets the output signal at the given index to `None`.
    #[inline]
    pub fn set_output_none(&mut self, index: usize) {
        self.outputs[index].as_mut().set_none();
    }

    /// Sets the output signal at the given index.
    #[inline]
    pub fn set_output(&mut self, index: usize, signal: AnySignal) -> Result<(), ProcessorError> {
        if signal.signal_type() != self.output_spec[index].signal_type {
            return Err(ProcessorError::OutputSpecMismatch {
                index,
                expected: self.output_spec[index].signal_type,
                actual: signal.signal_type(),
            });
        }

        self.outputs[index].clone_from_ref(signal.into_any_signal_opt().as_ref());

        Ok(())
    }

    /// Sets the output signal at the given index.
    #[inline]
    pub fn set_output_as<S: Signal>(
        &mut self,
        index: usize,
        signal: S,
    ) -> Result<(), ProcessorError> {
        let signal = signal.into_any_signal();

        if signal.signal_type() != self.output_spec[index].signal_type {
            return Err(ProcessorError::OutputSpecMismatch {
                index,
                expected: self.output_spec[index].signal_type,
                actual: signal.signal_type(),
            });
        }

        self.outputs[index].clone_from_ref(signal.into_any_signal_opt().as_ref());

        Ok(())
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
