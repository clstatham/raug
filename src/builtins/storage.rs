//! Storage-related processors.

use crate::prelude::*;

/// A processor that reads from and writes to a buffer of audio samples.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `index` | `Float` | The index of the sample to read. |
/// | `1` | `set` | `Float` | The value to write to the buffer at the current index. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Float` | The value of the sample at the current index. |
/// | `1` | `length` | `Int` | The length of the buffer. |
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AudioBuffer {
    buffer: Buffer<Float>,
    sample_rate: Float,
    index: Float,
}

impl AudioBuffer {
    /// Creates a new [`AudioBuffer`] processor with the given buffer.
    pub fn new(buffer: Buffer<Float>) -> Self {
        Self {
            buffer,
            sample_rate: 0.0,
            index: 0.0,
        }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for AudioBuffer {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("index", SignalType::Float),
            SignalSpec::new("set", SignalType::Float),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalType::Float)]
    }

    fn resize_buffers(&mut self, sample_rate: Float, _block_size: usize) {
        self.sample_rate = sample_rate;
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (out, index, write) in itertools::izip!(
            outputs.iter_output_mut_as_floats(0)?,
            inputs.iter_input_as_floats(0)?,
            inputs.iter_input_as_floats(1)?,
        ) {
            self.index = index.unwrap_or(self.index);

            if let Some(write) = write {
                self.buffer[self.index as usize] = Some(write);
            }

            if self.index.fract() != 0.0 {
                let pos_floor = self.index.floor() as usize;
                let pos_ceil = self.index.ceil() as usize;

                let value_floor = self.buffer[pos_floor].unwrap_or_default();
                let value_ceil = self.buffer[pos_ceil].unwrap_or_default();

                let t = self.index.fract();

                *out = Some(value_floor + (value_ceil - value_floor) * t);
            } else {
                let index = self.index as i64;

                if index < 0 {
                    self.index = self.buffer.len() as Float + index as Float;
                } else {
                    self.index = index as Float;
                }

                *out = Some(self.buffer[self.index as usize].unwrap_or_default());
            }
        }

        Ok(())
    }
}

/// A processor that stores / "remembers" a single value and outputs it continuously.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `set` | `Any` | The value to store. |
/// | `1` | `clear` | `Bool` | Whether to clear the stored value. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Any` | The stored value. |
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Register {
    value: AnySignal,
}

impl Register {
    /// Creates a new [`Register`] processor.
    pub fn new(signal_type: SignalType) -> Self {
        Self {
            value: AnySignal::default_of_type(&signal_type),
        }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for Register {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("set", self.value.signal_type()),
            SignalSpec::new("clear", SignalType::Bool),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", self.value.signal_type())]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (set, clear, mut out) in itertools::izip!(
            inputs.iter_input(0),
            inputs.iter_input_as_bools(1)?,
            outputs.iter_output(0),
        ) {
            if let Some(set) = set {
                self.value = set.to_owned();
            }

            if clear.is_some() {
                self.value.as_mut().set_none();
            }

            out.set(self.value.to_owned());
        }

        Ok(())
    }
}
