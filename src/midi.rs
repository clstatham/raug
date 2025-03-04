use std::ops::{Deref, DerefMut};

/// A 3-byte MIDI message.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MidiMessage {
    /// The MIDI message data.
    pub data: [u8; 3],
}

impl MidiMessage {
    /// Creates a new MIDI message from the given data.
    pub const fn new(data: [u8; 3]) -> Self {
        Self { data }
    }

    pub const fn note_on(channel: u8, note: u8, velocity: u8) -> Self {
        Self {
            data: [0x90 | (channel & 0x0F), note, velocity],
        }
    }

    pub const fn note_off(channel: u8, note: u8, velocity: u8) -> Self {
        Self {
            data: [0x80 | (channel & 0x0F), note, velocity],
        }
    }

    pub const fn control_change(channel: u8, control: u8, value: u8) -> Self {
        Self {
            data: [0xB0 | (channel & 0x0F), control, value],
        }
    }

    /// Returns the status byte of the MIDI message.
    pub const fn status(&self) -> u8 {
        self.data[0] & 0xF0
    }

    /// Returns the channel of the MIDI message.
    pub const fn channel(&self) -> u8 {
        self.data[0] & 0x0F
    }

    /// Returns the first data byte of the MIDI message.
    pub const fn data1(&self) -> u8 {
        self.data[1]
    }

    /// Returns the second data byte of the MIDI message.
    pub const fn data2(&self) -> u8 {
        self.data[2]
    }
}

impl Deref for MidiMessage {
    type Target = [u8; 3];
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for MidiMessage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
