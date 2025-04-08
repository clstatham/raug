//! Storage-related processors.

use crate::prelude::*;

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
