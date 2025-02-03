//! Built-in processors for MIDI messages.

use crate::prelude::*;

/// A processor that extracts the note number from a MIDI message.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `midi` | `Midi` | The input MIDI message. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `note` | `Float` | The note number of the input MIDI message. |
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MidiNote {
    note: Float,
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for MidiNote {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("midi", SignalType::Midi)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("note", SignalType::Float)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (midi, out) in iter_proc_io!(inputs as [MidiMessage], outputs as [Float]) {
            if let Some(msg) = midi {
                if msg.status() == 0x90 {
                    self.note = msg.data1() as Float;
                }
            }

            *out = Some(self.note);
        }
        Ok(())
    }
}

/// A processor that extracts the velocity from a MIDI message.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `midi` | `Midi` | The input MIDI message. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `velocity` | `Float` | The velocity of the input MIDI message. |
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MidiVelocity {
    velocity: Float,
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for MidiVelocity {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("midi", SignalType::Midi)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("velocity", SignalType::Float)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (midi, out) in iter_proc_io!(inputs as [MidiMessage], outputs as [Float]) {
            if let Some(msg) = midi {
                // Note on, note off, and polyphonic aftertouch messages.
                if [0x90, 0x80, 0xa8].contains(&msg.status()) {
                    self.velocity = msg.data2() as Float;
                }
            }

            *out = Some(self.velocity);
        }
        Ok(())
    }
}

/// A processor that outputs a gate signal from a MIDI note on/off message.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `midi` | `Midi` | The input MIDI message. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `gate` | `Bool` | The gate signal. |
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MidiGate {
    gate: bool,
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for MidiGate {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("midi", SignalType::Midi)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("gate", SignalType::Bool)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (midi, out) in iter_proc_io!(inputs as [MidiMessage], outputs as [bool]) {
            if let Some(msg) = midi {
                if msg.status() == 0x90 {
                    self.gate = msg.data2() > 0;
                } else if msg.status() == 0x80 {
                    self.gate = false;
                }
            }

            *out = Some(self.gate);
        }
        Ok(())
    }
}

/// A processor that outputs a trigger signal from a MIDI note on message.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `midi` | `Midi` | The input MIDI message. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `trigger` | `Bool` | The trigger signal. |
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MidiTrigger;

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for MidiTrigger {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("midi", SignalType::Midi)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("trigger", SignalType::Bool)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (midi, out) in iter_proc_io!(inputs as [MidiMessage], outputs as [bool]) {
            *out = None;
            if let Some(msg) = midi {
                if msg.status() == 0x90 && msg.data2() > 0 {
                    *out = Some(true);
                }
            }
        }
        Ok(())
    }
}

/// A processor that outputs the channel number from a MIDI message.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `midi` | `Midi` | The input MIDI message. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `channel` | `Float` | The channel number of the input MIDI message. |
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MidiChannel;

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for MidiChannel {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("midi", SignalType::Midi)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("channel", SignalType::Float)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (midi, out) in iter_proc_io!(inputs as [MidiMessage], outputs as [Float]) {
            *out = None;
            if let Some(msg) = midi {
                let channel = msg.channel() as Float;
                *out = Some(channel);
            }
        }
        Ok(())
    }
}
