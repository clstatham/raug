//! Built-in processors and utilities for the audio graph.

pub mod control;
pub mod dynamics;
pub mod filters;
pub mod math;
pub mod midi;
pub mod oscillators;
pub mod storage;
pub mod time;
pub mod util;

#[cfg(feature = "fft")]
pub mod simple_fft;

pub use control::*;
pub use dynamics::*;
pub use filters::*;
pub use math::*;
pub use midi::*;
pub use oscillators::*;
pub use storage::*;
pub use time::*;
pub use util::*;

#[cfg(feature = "fft")]
pub use simple_fft::*;

use crate::{prelude::*, runtime::RuntimeError};

/// Linear interpolation.
#[doc(hidden)]
#[inline]
pub fn lerp(a: Float, b: Float, t: Float) -> Float {
    debug_assert!((0.0..=1.0).contains(&t));
    a + (b - a) * t
}

/// A processor that runs a sub-graph.
///
/// # Inputs
///
/// The inputs of the sub-graph.
///
/// # Outputs
///
/// The outputs of the sub-graph.
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SubGraph {
    rt: Runtime,
}

impl SubGraph {
    /// Creates a new [`SubGraph`] processor with the given graph.
    pub fn new(graph: Graph) -> Self {
        Self {
            rt: Runtime::new(graph),
        }
    }

    pub fn build<F>(f: F) -> Self
    where
        F: FnOnce(&GraphBuilder),
    {
        let builder = GraphBuilder::new();
        f(&builder);
        Self::new(builder.build())
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for SubGraph {
    fn input_spec(&self) -> Vec<SignalSpec> {
        let mut spec = vec![];
        for (i, _input) in self.rt.graph().input_indices().iter().enumerate() {
            spec.push(SignalSpec::new(format!("{}", i), SignalType::Float));
        }
        spec
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        let mut spec = vec![];
        for (i, _output) in self.rt.graph().output_indices().iter().enumerate() {
            spec.push(SignalSpec::new(format!("{}", i), SignalType::Float));
        }
        spec
    }

    fn allocate(&mut self, sample_rate: Float, max_block_size: usize) {
        self.rt.allocate_for_block_size(sample_rate, max_block_size);
    }

    fn resize_buffers(&mut self, _sample_rate: Float, block_size: usize) {
        self.rt.set_block_size(block_size).unwrap();
    }

    fn num_inputs(&self) -> usize {
        self.rt.graph().input_indices().len()
    }

    fn num_outputs(&self) -> usize {
        self.rt.graph().output_indices().len()
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for i in 0..self.num_inputs() {
            let signal = inputs.input(i).ok_or(ProcessorError::NumInputsMismatch)?;
            let input = self
                .rt
                .get_input_mut(i)
                .ok_or(ProcessorError::NumInputsMismatch)?;
            input.clone_from(signal);
        }

        match self.rt.process() {
            Ok(()) => {}
            Err(RuntimeError::GraphRunError(e)) => {
                return Err(ProcessorError::SubGraph(Box::new(e)))
            }
            Err(_) => {
                return Err(ProcessorError::Other);
            }
        }

        for i in 0..self.num_outputs() {
            let output = self
                .rt
                .get_output(i)
                .ok_or(ProcessorError::NumOutputsMismatch)?;
            let mut signal = outputs.output(i);
            for i in 0..output.len() {
                signal.set(i, output.get(i).unwrap());
            }
        }

        Ok(())
    }
}
