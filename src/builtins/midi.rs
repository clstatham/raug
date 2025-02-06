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
pub struct MidiNote;

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
        for (midi, note) in iter_proc_io_as!(inputs as [MidiMessage], outputs as [Float]) {
            if let Some(msg) = midi {
                if msg.status() == 0x90 {
                    *note = Some(msg.data1() as Float);
                }
            }
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
pub struct MidiVelocity;

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
        for (midi, velocity) in iter_proc_io_as!(inputs as [MidiMessage], outputs as [Float]) {
            if let Some(msg) = midi {
                if msg.status() == 0x90 {
                    *velocity = Some(msg.data2() as Float);
                }
            }
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
        for (midi, gate) in iter_proc_io_as!(inputs as [MidiMessage], outputs as [bool]) {
            if let Some(msg) = midi {
                if msg.status() == 0x90 {
                    self.gate = msg.data2() > 0;
                } else if msg.status() == 0x80 {
                    self.gate = false;
                }

                *gate = Some(self.gate);
            }
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
        for (midi, trigger) in iter_proc_io_as!(inputs as [MidiMessage], outputs as [bool]) {
            if let Some(msg) = midi {
                if msg.status() == 0x90 && msg.data2() > 0 {
                    *trigger = Some(true);
                } else {
                    *trigger = Some(false);
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
        for (midi, channel) in iter_proc_io_as!(inputs as [MidiMessage], outputs as [Float]) {
            if let Some(msg) = midi {
                *channel = Some(msg.channel() as Float);
            }
        }

        Ok(())
    }
}
