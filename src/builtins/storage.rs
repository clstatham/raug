//! Storage-related processors.

use crate::{prelude::*, signal::optional::Repr};

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
#[derive(Clone, Debug, Processor)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", processor_typetag)]
pub struct AudioBuffer {
    buffer: String,
    #[input]
    index: Float,
    #[input]
    input: Float,
    #[input]
    write: bool,
    #[output]
    out: Float,
    #[output]
    length: i64,
}

impl AudioBuffer {
    /// Creates a new [`AudioBuffer`] processor with the given buffer.
    pub fn new(buffer: impl Into<String>) -> Self {
        Self {
            buffer: buffer.into(),
            index: 0.0,
            input: 0.0,
            write: false,
            out: 0.0,
            length: 0,
        }
    }

    fn update(&mut self, env: &ProcEnv) {
        let mut buffer = env.asset(&self.buffer).unwrap();
        let buffer = buffer.as_buffer_mut().unwrap();
        if self.write {
            buffer[self.index as usize].set(self.input);
        }

        if self.index.fract() != 0.0 {
            let pos_floor = self.index.floor() as usize;
            let pos_ceil = self.index.ceil() as usize;

            let value_floor = buffer[pos_floor].unwrap_or_default().into_signal();
            let value_ceil = buffer[pos_ceil].unwrap_or_default().into_signal();

            let t = self.index.fract();

            self.out = value_floor + (value_ceil - value_floor) * t;
        } else {
            let index = self.index as i64;

            if index < 0 {
                self.index = buffer.len() as Float + index as Float;
            } else {
                self.index = index as Float;
            }

            self.out = buffer[self.index as usize]
                .unwrap_or_default()
                .into_signal();
        }

        self.length = buffer.len() as i64;
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
        for (set, clear, mut out) in iter_proc_io_as!(inputs as [Any, bool], outputs as [Any]) {
            if let Some(set) = set {
                self.value = set;
            }

            if clear.unwrap_or_default() {
                self.value.set_none();
            }

            if self.value.is_some() {
                out.set_any_opt(self.value);
            }
        }

        Ok(())
    }
}
