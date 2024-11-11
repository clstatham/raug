//! A module for working with messages that can be sent between processors.

use std::fmt::{Debug, Display};

use crate::signal::Sample;

/// A message that can be sent between processors.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Message {
    /// A bang message ("do whatever it is you do").
    Bang,
    /// A boolean message.
    Bool(bool),
    /// An integer message.
    Int(i64),
    /// A float message.
    Float(Sample),
    /// A string message.
    String(String),
    /// A list of messages.
    List(Vec<Message>),
    /// A MIDI message.
    Midi(Vec<u8>),
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Message::Bang => write!(f, "bang"),
            Message::Bool(b) => write!(f, "{}", b),
            Message::Int(i) => write!(f, "{}", i),
            Message::Float(x) => write!(f, "{}", x),
            Message::String(s) => write!(f, "{}", s),
            Message::List(list) => {
                write!(f, "[")?;
                for (i, item) in list.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            Message::Midi(data) => {
                write!(f, "MIDI(")?;
                for (i, byte) in data.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{:02X}", byte)?;
                }
                write!(f, ")")
            }
        }
    }
}

impl Message {
    /// Returns true if the two messages are of the same message type.
    #[inline]
    pub fn is_same_type(&self, other: &Message) -> bool {
        matches!(
            (self, other),
            (Message::Bang, Message::Bang)
                | (Message::Bool(_), Message::Bool(_))
                | (Message::Int(_), Message::Int(_))
                | (Message::Float(_), Message::Float(_))
                | (Message::String(_), Message::String(_))
                | (Message::List(_), Message::List(_))
                | (Message::Midi(_), Message::Midi(_))
        )
    }

    /// Attempts to convert the message to a boolean.
    ///
    /// This does not attempt to *cast* the message to a boolean, but rather checks if the message is already `Message::Bool`.
    #[inline]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Message::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Attempts to convert the message to an integer.
    ///
    /// This does not attempt to *cast* the message to an integer, but rather checks if the message is already `Message::Int`.
    #[inline]
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Message::Int(i) => Some(*i),
            _ => None,
        }
    }

    /// Attempts to convert the message to a float.
    ///
    /// This does not attempt to *cast* the message to a float, but rather checks if the message is already `Message::Float`.
    #[inline]
    pub fn as_float(&self) -> Option<Sample> {
        match self {
            Message::Float(x) => Some(*x),
            _ => None,
        }
    }

    /// Attempts to convert the message to a string.
    ///
    /// This does not attempt to *cast* the message to a string, but rather checks if the message is already `Message::String`.
    #[inline]
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Message::String(s) => Some(s),
            _ => None,
        }
    }

    /// Attempts to convert the message to a list.
    ///
    /// This does not attempt to *cast* the message to a list, but rather checks if the message is already `Message::List`.
    #[inline]
    pub fn as_list(&self) -> Option<&Vec<Message>> {
        match self {
            Message::List(list) => Some(list),
            _ => None,
        }
    }

    /// Attempts to convert the message to a MIDI message.
    ///
    /// This does not attempt to *cast* the message to a MIDI message, but rather checks if the message is already `Message::Midi`.
    #[inline]
    pub fn as_midi(&self) -> Option<&[u8]> {
        match self {
            Message::Midi(data) => Some(data),
            _ => None,
        }
    }

    /// Returns true if the message is a bang.
    #[inline]
    pub fn is_bang(&self) -> bool {
        matches!(self, Message::Bang)
    }

    /// Returns true if the message is a boolean.
    #[inline]
    pub fn is_bool(&self) -> bool {
        matches!(self, Message::Bool(_))
    }

    /// Returns true if the message is an integer.
    #[inline]
    pub fn is_int(&self) -> bool {
        matches!(self, Message::Int(_))
    }

    /// Returns true if the message is a float.
    #[inline]
    pub fn is_float(&self) -> bool {
        matches!(self, Message::Float(_))
    }

    /// Returns true if the message is a string.
    #[inline]
    pub fn is_string(&self) -> bool {
        matches!(self, Message::String(_))
    }

    /// Returns true if the message is a list.
    #[inline]
    pub fn is_list(&self) -> bool {
        matches!(self, Message::List(_))
    }

    /// Returns true if the message is a MIDI message.
    #[inline]
    pub fn is_midi(&self) -> bool {
        matches!(self, Message::Midi(_))
    }

    /// Attempts to cast the message to an integer using whatever method is most appropriate.
    ///
    /// Currently, this is defined as:
    /// - `Message::Bool` is returned as `1` if `true`, `0` if `false`.
    /// - `Message::Int` is returned as-is.
    /// - `Message::Float` is rounded to the nearest integer.
    /// - `Message::String` is parsed as an integer.
    /// - All other types return `None`.
    #[inline]
    pub fn cast_to_int(&self) -> Option<i64> {
        match self {
            Message::Int(i) => Some(*i),
            Message::Bool(b) => Some(if *b { 1 } else { 0 }),
            Message::Float(x) => Some(x.round() as i64),
            Message::String(s) => s.parse().ok(),
            _ => None,
        }
    }

    /// Attempts to cast the message to a boolean using whatever method is most appropriate.
    ///
    /// Currently, this is defined as:
    /// - `Message::Bool` is returned as-is.
    /// - `Message::Int` is returned as `true` if not zero, `false` if zero.
    /// - `Message::Float` is returned as `true` if not zero, `false` if zero.
    /// - `Message::String` is returned as `true` if not empty, `false` if empty.
    /// - `Message::List` is returned as `true` if not empty, `false` if empty.
    /// - `Message::Midi` is returned as `true` if not empty, `false` if empty.
    /// - All other types return `None`.
    #[inline]
    pub fn cast_to_bool(&self) -> Option<bool> {
        match self {
            Message::Bool(b) => Some(*b),
            Message::Int(i) => Some(*i != 0),
            Message::Float(x) => Some(*x != 0.0),
            Message::String(s) => Some(!s.is_empty()),
            Message::List(list) => Some(!list.is_empty()),
            Message::Midi(data) => Some(!data.is_empty()),
            _ => None,
        }
    }

    /// Attempts to cast the message to a float using whatever method is most appropriate.
    ///
    /// Currently, this is defined as:
    /// - `Message::Bool` is returned as `1.0` if `true`, `0.0` if `false`.
    /// - `Message::Int` is returned as-is, but converted to a float.
    /// - `Message::Float` is returned as-is.
    /// - `Message::String` is parsed as a float.
    /// - All other types return `None`.
    #[inline]
    pub fn cast_to_float(&self) -> Option<Sample> {
        match self {
            Message::Int(i) => Some(*i as Sample),
            Message::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
            Message::Float(x) => Some(*x),
            Message::String(s) => s.parse().ok(),
            _ => None,
        }
    }

    /// Attempts to cast the message to a string using whatever method is most appropriate.
    ///
    /// Currently, this is defined as:
    /// - `Message::Bang` is returned as `"bang"`.
    /// - `Message::Bool` is returned as `"true"` or `"false"`.
    /// - `Message::Int` is returned as the integer converted to a string.
    /// - `Message::Float` is returned as the float converted to a string.
    /// - `Message::String` is returned as-is.
    /// - All other types return `None`.
    #[inline]
    pub fn cast_to_string(&self) -> Option<String> {
        match self {
            Message::Bang => Some("bang".to_string()),
            Message::Bool(b) => Some(b.to_string()),
            Message::Int(i) => Some(i.to_string()),
            Message::Float(x) => Some(x.to_string()),
            Message::String(s) => Some(s.to_string()),
            _ => None,
        }
    }
}

impl From<i64> for Message {
    fn from(i: i64) -> Self {
        Message::Int(i)
    }
}

impl From<Sample> for Message {
    fn from(x: Sample) -> Self {
        Message::Float(x)
    }
}

impl From<&str> for Message {
    fn from(s: &str) -> Self {
        Message::String(String::from(s))
    }
}

impl From<String> for Message {
    fn from(s: String) -> Self {
        Message::String(s)
    }
}

impl From<Vec<Message>> for Message {
    fn from(list: Vec<Message>) -> Self {
        Message::List(list)
    }
}

impl From<Vec<u8>> for Message {
    fn from(data: Vec<u8>) -> Self {
        Message::Midi(data)
    }
}
