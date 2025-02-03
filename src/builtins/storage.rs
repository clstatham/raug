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
    buffer: String,
    index: Float,
}

impl AudioBuffer {
    /// Creates a new [`AudioBuffer`] processor with the given buffer.
    pub fn new(buffer: impl Into<String>) -> Self {
        Self {
            buffer: buffer.into(),
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
        vec![
            SignalSpec::new("out", SignalType::Float),
            SignalSpec::new("length", SignalType::Int),
        ]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        let buffer = inputs.asset(&self.buffer)?;
        let mut buffer = buffer.try_lock().unwrap();
        let buffer = buffer.as_buffer_mut().ok_or_else(|| {
            ProcessorError::InvalidAsset(self.buffer.clone(), "Buffer".to_string())
        })?;
        for (index, write, out, length) in iter_proc_io!(
            inputs as [Float, Float],
            outputs as [Float, i64]
        ) {
            self.index = index.unwrap_or(self.index);

            if let Some(write) = *write {
                buffer[self.index as usize] = Some(write);
            }

            if self.index.fract() != 0.0 {
                let pos_floor = self.index.floor() as usize;
                let pos_ceil = self.index.ceil() as usize;

                let value_floor = buffer[pos_floor].unwrap_or_default();
                let value_ceil = buffer[pos_ceil].unwrap_or_default();

                let t = self.index.fract();

                *out = Some(value_floor + (value_ceil - value_floor) * t);
            } else {
                let index = self.index as i64;

                if index < 0 {
                    self.index = self.buffer.len() as Float + index as Float;
                } else {
                    self.index = index as Float;
                }

                *out = Some(buffer[self.index as usize].unwrap_or_default());
            }

            *length = Some(buffer.len() as i64);
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
    value: AnySignalOpt,
}

impl Register {
    /// Creates a new [`Register`] processor.
    pub fn new(signal_type: SignalType) -> Self {
        Self {
            value: AnySignalOpt::default_of_type(&signal_type),
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
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (set, clear, mut out) in iter_proc_io!(
            inputs as [Any, bool],
            outputs as [Any]
        ) {
            if let Some(set) = set {
                self.value.clone_from_ref(set);
            }

            if clear.is_some() {
                self.value.as_mut().set_none();
            }

            out.clone_from_opt_ref(self.value.as_ref());
        }

        Ok(())
    }
}
