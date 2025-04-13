//! Audio processing utilities and types.

use std::fmt::Debug;

use downcast_rs::{Downcast, impl_downcast};
use io::{ProcessorInputs, ProcessorOutputs, SignalSpec};
use thiserror::Error;

use crate::{prelude::ErasedBuffer, signal::SignalType};

pub mod io;

/// Error type for [`Processor`] operations.
#[derive(Debug, Error)]
pub enum ProcessorError {
    /// Input signal type mismatch.
    #[error("Input {index} signal type mismatch (expected {expected:?}, got {actual:?})")]
    InputTypeMismatch {
        /// The index of the input signal.
        index: usize,
        /// The expected signal type.
        expected: SignalType,
        /// The actual signal type.
        actual: SignalType,
    },

    /// Output signal type mismatch.
    #[error("Output {index} signal type mismatch (expected {expected:?}, got {actual:?})")]
    OutputTypeMismatch {
        /// The index of the output signal.
        index: usize,
        /// The expected signal type.
        expected: SignalType,
        /// The actual signal type.
        actual: SignalType,
    },
}

/// A result type for processor operations.
pub type ProcResult<T> = Result<T, ProcessorError>;

/// A processor that can process audio signals.
pub trait Processor
where
    Self: Downcast + ProcessorClone + Send,
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

    /// Creates a new set of output buffers for the processor.
    fn create_output_buffers(&self, size: usize) -> Vec<ErasedBuffer>;

    /// Called once, before processing starts.
    ///
    /// Do all of your preallocation here.
    #[allow(unused)]
    fn allocate(&mut self, sample_rate: f32, max_block_size: usize) {}

    /// Called anytime the sample rate or block size changes.
    ///
    /// This function is NOT ALLOWED to allocate memory.
    #[allow(unused)]
    fn resize_buffers(&mut self, sample_rate: f32, block_size: usize) {}

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
