//! Signal types and operations.

use std::fmt::Debug;

use buffer::Buffer;
use optional::{FloatRepr, Repr};

use crate::midi::MidiMessage;

pub mod buffer;
pub mod optional;

#[cfg(feature = "f32_samples")]
/// The floating-point sample type.
pub type Float = f32;
#[cfg(not(feature = "f32_samples"))]
/// The floating-point sample type.
pub type Float = f64;

#[cfg(feature = "f32_samples")]
/// The value of PI for the floating-point sample type.
pub const PI: Float = std::f32::consts::PI;
/// The value of PI for the floating-point sample type.
#[cfg(not(feature = "f32_samples"))]
pub const PI: Float = std::f64::consts::PI;

#[cfg(feature = "f32_samples")]
/// The value of TAU (2*PI) for the floating-point sample type.
pub const TAU: Float = std::f32::consts::TAU;
#[cfg(not(feature = "f32_samples"))]
/// The value of TAU (2*PI) for the floating-point sample type.
pub const TAU: Float = std::f64::consts::TAU;

mod sealed {
    use crate::midi::MidiMessage;

    pub trait Sealed {}
    impl Sealed for f32 {}
    impl Sealed for f64 {}
    impl Sealed for i64 {}
    impl Sealed for bool {}
    impl Sealed for MidiMessage {}
}

/// A type that can be stored in a [`Buffer`] and processed by a [`Processor`](crate::processor::Processor).
pub trait Signal: sealed::Sealed + Copy + Debug + Send + Sync + PartialEq + 'static {
    type Repr: Repr<Self> + Copy + Debug + Send + Sync + PartialEq + 'static;

    fn from_repr(repr: Self::Repr) -> Self;
    fn into_repr(self) -> Self::Repr;

    /// The type of the signal.
    fn signal_type() -> SignalType;

    /// Converts the signal into an [`AnySignal`].
    fn into_any_signal(self) -> AnySignal;

    /// Attempts to convert an [`AnySignal`] into the signal type.
    fn try_from_any_signal(signal: AnySignal) -> Option<Self>;

    /// Attempts to convert a [`SignalBuffer`] into a buffer of the signal type.
    fn try_convert_buffer(buffer: &SignalBuffer) -> Option<&Buffer<Self>>;

    /// Attempts to convert a mutable [`SignalBuffer`] into a mutable buffer of the signal type.
    fn try_convert_buffer_mut(buffer: &mut SignalBuffer) -> Option<&mut Buffer<Self>>;
}

pub type OptionRepr<T> = Option<<T as Signal>::Repr>;

pub trait OptSignal<T: Signal> {
    fn into_repr(self) -> Option<T::Repr>;
    fn set_repr(&mut self, value: Option<T::Repr>);
    fn set_none(&mut self);
}

impl<T: Signal> OptSignal<T> for Option<T> {
    #[inline]
    fn into_repr(self) -> Option<T::Repr> {
        self.map(|v| v.into_repr())
    }

    #[inline]
    fn set_repr(&mut self, value: Option<T::Repr>) {
        *self = value.map(T::from_repr);
    }

    #[inline]
    fn set_none(&mut self) {
        *self = None;
    }
}

pub trait OptRepr<T: Signal> {
    fn into_signal(self) -> Option<T>;
    fn set(&mut self, value: T);
}

impl<T: Signal> OptRepr<T> for Option<T::Repr> {
    #[inline]
    fn into_signal(self) -> Option<T> {
        self.map(|v| T::from_repr(v))
    }

    #[inline]
    fn set(&mut self, value: T) {
        *self = Some(value.into_repr());
    }
}

impl Signal for Float {
    type Repr = FloatRepr;

    fn from_repr(repr: Self::Repr) -> Self {
        repr.into_signal()
    }

    fn into_repr(self) -> Self::Repr {
        // FloatRepr::new(self)
        Self::Repr::from_signal(self)
    }

    #[inline]
    fn signal_type() -> SignalType {
        SignalType::Float
    }

    #[inline]
    fn into_any_signal(self) -> AnySignal {
        AnySignal::Float(self)
    }

    #[inline]
    fn try_from_any_signal(signal: AnySignal) -> Option<Self> {
        match signal {
            AnySignal::Float(float) => Some(float),
            _ => None,
        }
    }

    #[inline]
    fn try_convert_buffer(buffer: &SignalBuffer) -> Option<&Buffer<Self>> {
        match buffer {
            SignalBuffer::Float(buffer) => Some(buffer),
            _ => None,
        }
    }

    #[inline]
    fn try_convert_buffer_mut(buffer: &mut SignalBuffer) -> Option<&mut Buffer<Self>> {
        match buffer {
            SignalBuffer::Float(buffer) => Some(buffer),
            _ => None,
        }
    }
}

impl Signal for i64 {
    type Repr = i64;

    fn from_repr(repr: Self::Repr) -> Self {
        repr
    }

    fn into_repr(self) -> Self::Repr {
        self
    }

    #[inline]
    fn signal_type() -> SignalType {
        SignalType::Int
    }

    #[inline]
    fn into_any_signal(self) -> AnySignal {
        AnySignal::Int(self)
    }

    #[inline]
    fn try_from_any_signal(signal: AnySignal) -> Option<Self> {
        match signal {
            AnySignal::Int(int) => Some(int),
            _ => None,
        }
    }

    #[inline]
    fn try_convert_buffer(buffer: &SignalBuffer) -> Option<&Buffer<Self>> {
        match buffer {
            SignalBuffer::Int(buffer) => Some(buffer),
            _ => None,
        }
    }

    #[inline]
    fn try_convert_buffer_mut(buffer: &mut SignalBuffer) -> Option<&mut Buffer<Self>> {
        match buffer {
            SignalBuffer::Int(buffer) => Some(buffer),
            _ => None,
        }
    }
}

impl Signal for bool {
    type Repr = bool;

    fn from_repr(repr: Self::Repr) -> Self {
        repr
    }

    fn into_repr(self) -> Self::Repr {
        self
    }

    #[inline]
    fn signal_type() -> SignalType {
        SignalType::Bool
    }

    #[inline]
    fn into_any_signal(self) -> AnySignal {
        AnySignal::Bool(self)
    }

    #[inline]
    fn try_from_any_signal(signal: AnySignal) -> Option<Self> {
        match signal {
            AnySignal::Bool(bool) => Some(bool),
            _ => None,
        }
    }

    #[inline]
    fn try_convert_buffer(buffer: &SignalBuffer) -> Option<&Buffer<Self>> {
        match buffer {
            SignalBuffer::Bool(buffer) => Some(buffer),
            _ => None,
        }
    }

    #[inline]
    fn try_convert_buffer_mut(buffer: &mut SignalBuffer) -> Option<&mut Buffer<Self>> {
        match buffer {
            SignalBuffer::Bool(buffer) => Some(buffer),
            _ => None,
        }
    }
}

impl Signal for MidiMessage {
    type Repr = MidiMessage;

    fn from_repr(repr: Self::Repr) -> Self {
        repr
    }

    fn into_repr(self) -> Self::Repr {
        self
    }

    #[inline]
    fn signal_type() -> SignalType {
        SignalType::Midi
    }

    #[inline]
    fn into_any_signal(self) -> AnySignal {
        AnySignal::Midi(self)
    }

    #[inline]
    fn try_from_any_signal(signal: AnySignal) -> Option<Self> {
        match signal {
            AnySignal::Midi(midi) => Some(midi),
            _ => None,
        }
    }

    #[inline]
    fn try_convert_buffer(buffer: &SignalBuffer) -> Option<&Buffer<Self>> {
        match buffer {
            SignalBuffer::Midi(buffer) => Some(buffer),
            _ => None,
        }
    }

    #[inline]
    fn try_convert_buffer_mut(buffer: &mut SignalBuffer) -> Option<&mut Buffer<Self>> {
        match buffer {
            SignalBuffer::Midi(buffer) => Some(buffer),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AnySignal {
    Float(Float),
    Int(i64),
    Bool(bool),
    Midi(MidiMessage),
}

impl AnySignal {
    /// Returns the type of the signal.
    #[inline]
    pub fn signal_type(&self) -> SignalType {
        match self {
            Self::Float(_) => SignalType::Float,
            Self::Int(_) => SignalType::Int,
            Self::Bool(_) => SignalType::Bool,
            Self::Midi(_) => SignalType::Midi,
        }
    }

    #[inline]
    pub fn default_of_type(signal_type: &SignalType) -> Self {
        match signal_type {
            SignalType::Float => AnySignal::Float(0.0),
            SignalType::Int => AnySignal::Int(0),
            SignalType::Bool => AnySignal::Bool(false),
            SignalType::Midi => AnySignal::Midi(MidiMessage::new([0, 0, 0])),
        }
    }

    /// Attempts to cast the signal to the given signal type.
    ///
    /// Currently, the following conversions are supported:
    ///
    /// | From \ To | Float | Int | Bool | Midi |
    /// |-----------|-------|-----|------|------|
    /// | Float     | -     | Yes | Yes  | -    |
    /// | Int       | Yes   | -   | Yes  | -    |
    /// | Bool      | Yes   | Yes | -    | -    |
    /// | Midi      | -     | -   | -    | -    |
    #[inline]
    pub fn cast(&self, target: SignalType) -> Option<Self> {
        if self.signal_type() == target {
            return Some(*self);
        }
        match (self, target) {
            (Self::Float(float), SignalType::Int) => Some(Self::Int(*float as i64)),
            (Self::Float(float), SignalType::Bool) => Some(Self::Bool(*float != 0.0)),
            (Self::Int(int), SignalType::Float) => Some(Self::Float(*int as Float)),
            (Self::Int(int), SignalType::Bool) => Some(Self::Bool(*int != 0)),
            (Self::Bool(bool), SignalType::Float) => {
                Some(Self::Float(if *bool { 1.0 } else { 0.0 }))
            }
            (Self::Bool(bool), SignalType::Int) => Some(Self::Int(if *bool { 1 } else { 0 })),
            _ => None,
        }
    }

    /// Converts the signal into an [`AnySignalOpt`].
    /// The signal is wrapped in `Some`.
    #[inline]
    pub fn into_any_signal_opt(self) -> AnySignalOpt {
        match self {
            Self::Float(float) => AnySignalOpt::Float(Some(float)),
            Self::Int(int) => AnySignalOpt::Int(Some(int)),
            Self::Bool(bool) => AnySignalOpt::Bool(Some(bool)),
            Self::Midi(midi) => AnySignalOpt::Midi(Some(midi)),
        }
    }

    /// Attempts to extract the signal as the given signal type.
    /// Returns `None` if the signal type does not match.
    #[inline]
    pub fn as_type<T: Signal>(&self) -> Option<T> {
        T::try_from_any_signal(*self)
    }
}

/// A type that holds an Option containing any signal type.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AnySignalOpt {
    /// A floating-point value.
    Float(Option<Float>),

    /// An integer.
    Int(Option<i64>),

    /// A boolean.
    Bool(Option<bool>),

    /// A MIDI message.
    Midi(Option<MidiMessage>),
}

impl AnySignalOpt {
    /// Creates a new signal of the given type with no value.
    pub fn default_of_type(signal_type: &SignalType) -> Self {
        match signal_type {
            SignalType::Float => AnySignalOpt::Float(None),
            SignalType::Int => AnySignalOpt::Int(None),
            SignalType::Bool => AnySignalOpt::Bool(None),
            SignalType::Midi => AnySignalOpt::Midi(None),
        }
    }

    /// Returns `true` if the signal is `Some`.
    pub fn is_some(&self) -> bool {
        match self {
            Self::Float(float) => float.is_some(),
            Self::Int(int) => int.is_some(),
            Self::Bool(bool) => bool.is_some(),
            Self::Midi(midi) => midi.is_some(),
        }
    }

    /// Returns `true` if the signal is `None`.
    pub fn is_none(&self) -> bool {
        !self.is_some()
    }

    /// Returns `true` if the signal is of the given type.
    pub fn is_type<T: Signal>(&self) -> bool {
        self.signal_type() == T::signal_type()
    }

    /// Returns `true` if the signal is of the same type as the other signal.
    pub fn is_same_type(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::Float(_), Self::Float(_))
                | (Self::Int(_), Self::Int(_))
                | (Self::Bool(_), Self::Bool(_))
                | (Self::Midi(_), Self::Midi(_))
        )
    }

    /// Returns the type of the signal.
    #[inline]
    pub fn signal_type(&self) -> SignalType {
        match self {
            Self::Float(_) => SignalType::Float,
            Self::Int(_) => SignalType::Int,
            Self::Bool(_) => SignalType::Bool,
            Self::Midi(_) => SignalType::Midi,
        }
    }

    #[inline]
    pub fn as_type<T: Signal>(&self) -> Option<T> {
        self.try_into_any_signal().and_then(T::try_from_any_signal)
    }

    /// Attempts to cast the signal to the given signal type.
    ///
    /// Currently, the following conversions are supported:
    ///
    /// | From \ To | Float | Int | Bool | Midi |
    /// |-----------|-------|-----|------|------|
    /// | Float     | -     | Yes | Yes  | -    |
    /// | Int       | Yes   | -   | Yes  | -    |
    /// | Bool      | Yes   | Yes | -    | -    |
    /// | Midi      | -     | -   | -    | -    |
    #[inline]
    pub fn cast(&self, target: SignalType) -> Option<Self> {
        if self.signal_type() == target {
            return Some(*self);
        }
        match (self, target) {
            (Self::Float(float), SignalType::Int) => float.map(|f| Self::Int(Some(f as i64))),
            (Self::Float(float), SignalType::Bool) => float.map(|f| Self::Bool(Some(f != 0.0))),
            (Self::Int(int), SignalType::Float) => int.map(|i| Self::Float(Some(i as Float))),
            (Self::Int(int), SignalType::Bool) => int.map(|i| Self::Bool(Some(i != 0))),
            (Self::Bool(bool), SignalType::Float) => {
                bool.map(|b| Self::Float(Some(if b { 1.0 } else { 0.0 })))
            }
            (Self::Bool(bool), SignalType::Int) => {
                bool.map(|b| Self::Int(Some(if b { 1 } else { 0 })))
            }

            _ => None,
        }
    }

    /// Attempts to convert the signal into an [`AnySignal`].
    #[inline]
    pub fn try_into_any_signal(self) -> Option<AnySignal> {
        match self {
            Self::Float(float) => float.map(AnySignal::Float),
            Self::Int(int) => int.map(AnySignal::Int),
            Self::Bool(bool) => bool.map(AnySignal::Bool),
            Self::Midi(midi) => midi.map(AnySignal::Midi),
        }
    }

    #[inline]
    pub fn set_none(&mut self) {
        match self {
            Self::Float(float) => *float = None,
            Self::Int(int) => *int = None,
            Self::Bool(bool) => *bool = None,
            Self::Midi(midi) => *midi = None,
        }
    }
}

pub enum AnySignalOptMut<'a> {
    Float(&'a mut OptionRepr<Float>),
    Int(&'a mut OptionRepr<i64>),
    Bool(&'a mut OptionRepr<bool>),
    Midi(&'a mut OptionRepr<MidiMessage>),
}

impl AnySignalOptMut<'_> {
    /// Returns the type of the signal.
    #[inline]
    pub fn signal_type(&self) -> SignalType {
        match self {
            Self::Float(_) => SignalType::Float,
            Self::Int(_) => SignalType::Int,
            Self::Bool(_) => SignalType::Bool,
            Self::Midi(_) => SignalType::Midi,
        }
    }

    /// Returns `true` if the signal is `Some`.
    #[inline]
    pub fn is_some(&self) -> bool {
        match self {
            Self::Float(float) => float.is_some(),
            Self::Int(int) => int.is_some(),
            Self::Bool(bool) => bool.is_some(),
            Self::Midi(midi) => midi.is_some(),
        }
    }

    /// Returns `true` if the signal is `None`.
    #[inline]
    pub fn is_none(&self) -> bool {
        !self.is_some()
    }

    /// Returns `true` if the signal is of the given type.
    #[inline]
    pub fn is_type<T: Signal>(&self) -> bool {
        self.signal_type() == T::signal_type()
    }

    /// Returns `true` if the signal is of the same type as the other signal.
    #[inline]
    pub fn is_same_type(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::Float(_), Self::Float(_))
                | (Self::Int(_), Self::Int(_))
                | (Self::Bool(_), Self::Bool(_))
                | (Self::Midi(_), Self::Midi(_))
        )
    }

    #[inline]
    pub fn set<T: Signal>(&mut self, value: T) {
        let value = value.into_any_signal();
        self.set_any(value);
    }

    #[inline]
    pub fn set_any(&mut self, value: AnySignal) {
        let value = value.into_any_signal_opt();
        self.set_any_opt(value);
    }

    #[inline]
    pub fn set_any_opt(&mut self, value: AnySignalOpt) {
        match (self, value) {
            (Self::Float(float), AnySignalOpt::Float(value)) => **float = value.into_repr(),
            (Self::Int(int), AnySignalOpt::Int(value)) => **int = value.into_repr(),
            (Self::Bool(bool), AnySignalOpt::Bool(value)) => **bool = value.into_repr(),
            (Self::Midi(midi), AnySignalOpt::Midi(value)) => **midi = value.into_repr(),
            _ => panic!("Cannot set signal of different type"),
        }
    }

    #[inline]
    pub fn set_none(&mut self) {
        match self {
            Self::Float(float) => **float = None,
            Self::Int(int) => **int = None,
            Self::Bool(bool) => **bool = None,
            Self::Midi(midi) => **midi = None,
        }
    }
}

/// A signal type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SignalType {
    /// A floating-point signal.
    Float,

    /// An integer signal.
    Int,

    /// A boolean signal.
    Bool,

    /// A MIDI signal.
    Midi,
}

impl SignalType {
    /// Returns `true` if the signal type is compatible with the other signal type.
    #[inline]
    pub fn is_same_as(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::Float, Self::Float)
                | (Self::Int, Self::Int)
                | (Self::Bool, Self::Bool)
                | (Self::Midi, Self::Midi)
        )
    }
}

/// A buffer of signals that can hold any signal type.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SignalBuffer {
    /// A buffer of floating-point signals.
    Float(Buffer<Float>),

    /// A buffer of integer signals.
    Int(Buffer<i64>),

    /// A buffer of boolean signals.
    Bool(Buffer<bool>),

    /// A buffer of MIDI signals.
    Midi(Buffer<MidiMessage>),
}

impl SignalBuffer {
    /// Creates a new buffer of the given type with the given length filled with `None`.
    pub fn new_of_type(signal_type: &SignalType, length: usize) -> Self {
        match signal_type {
            SignalType::Float => Self::Float(Buffer::zeros(length)),
            SignalType::Int => Self::Int(Buffer::zeros(length)),
            SignalType::Bool => Self::Bool(Buffer::zeros(length)),
            SignalType::Midi => Self::Midi(Buffer::zeros(length)),
        }
    }

    /// Returns the type of the buffer.
    #[inline]
    pub fn signal_type(&self) -> SignalType {
        match self {
            Self::Float(_) => SignalType::Float,
            Self::Int(_) => SignalType::Int,
            Self::Bool(_) => SignalType::Bool,
            Self::Midi(_) => SignalType::Midi,
        }
    }

    /// Returns `true` if the buffer is of the given type.
    #[inline]
    pub fn is_type(&self, signal_type: SignalType) -> bool {
        self.signal_type() == signal_type
    }

    /// Returns a reference to the buffer as a buffer of the given signal type, if it is of that type.
    #[inline]
    pub fn as_type<S: Signal>(&self) -> Option<&Buffer<S>> {
        S::try_convert_buffer(self)
    }

    /// Returns a mutable reference to the buffer as a buffer of the given signal type, if it is of that type.
    #[inline]
    pub fn as_type_mut<S: Signal>(&mut self) -> Option<&mut Buffer<S>> {
        S::try_convert_buffer_mut(self)
    }

    /// Returns the length of the buffer.
    #[inline]
    pub fn len(&self) -> usize {
        match self {
            Self::Float(buffer) => buffer.len(),
            Self::Int(buffer) => buffer.len(),
            Self::Bool(buffer) => buffer.len(),
            Self::Midi(buffer) => buffer.len(),
        }
    }

    /// Returns `true` if the buffer is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Resizes the buffer to the given length, filling the new elements with the given value.
    ///
    /// # Panics
    ///
    /// Panics if the value type does not match the buffer type.
    pub fn resize(&mut self, length: usize, value: impl Into<AnySignalOpt>) {
        let value = value.into();
        match (self, value) {
            (Self::Float(buffer), AnySignalOpt::Float(value)) => {
                buffer.resize(length, value.into_repr())
            }
            (Self::Int(buffer), AnySignalOpt::Int(value)) => {
                buffer.resize(length, value.into_repr())
            }
            (Self::Bool(buffer), AnySignalOpt::Bool(value)) => {
                buffer.resize(length, value.into_repr())
            }
            (Self::Midi(buffer), AnySignalOpt::Midi(value)) => {
                buffer.resize(length, value.into_repr())
            }
            _ => panic!("Cannot resize buffer with value of different type"),
        }
    }

    /// Fills the buffer with the given value.
    ///
    /// # Panics
    ///
    /// Panics if the value type does not match the buffer type.
    pub fn fill(&mut self, value: impl Into<AnySignalOpt>) {
        let value = value.into();
        match (self, value) {
            (Self::Float(buffer), AnySignalOpt::Float(value)) => buffer.fill(value.into_repr()),
            (Self::Int(buffer), AnySignalOpt::Int(value)) => buffer.fill(value.into_repr()),
            (Self::Bool(buffer), AnySignalOpt::Bool(value)) => buffer.fill(value.into_repr()),
            (Self::Midi(buffer), AnySignalOpt::Midi(value)) => buffer.fill(value.into_repr()),
            _ => panic!("Cannot fill buffer with value of different type"),
        }
    }

    /// Resizes the buffer to the given length, filling the new elements with `None`.
    pub fn resize_default(&mut self, length: usize) {
        match self {
            Self::Float(buffer) => buffer.resize(length, None),
            Self::Int(buffer) => buffer.resize(length, None),
            Self::Bool(buffer) => buffer.resize(length, None),
            Self::Midi(buffer) => buffer.resize(length, None),
        }
    }

    /// Resizes the buffer based on the given type hint.
    pub fn resize_with_hint(&mut self, length: usize, type_hint: &SignalType) {
        let signal_type = self.signal_type();
        if signal_type.is_same_as(type_hint) {
            self.resize_default(length);
        } else {
            *self = Self::new_of_type(type_hint, length);
        }
    }

    /// Fills the buffer with `None`.
    pub fn fill_none(&mut self) {
        match self {
            Self::Float(buffer) => buffer.fill(None),
            Self::Int(buffer) => buffer.fill(None),
            Self::Bool(buffer) => buffer.fill(None),
            Self::Midi(buffer) => buffer.fill(None),
        }
    }

    /// Fills the buffer based on the given type hint.
    pub fn fill_with_hint(&mut self, type_hint: &SignalType) {
        let signal_type = self.signal_type();
        if signal_type.is_same_as(type_hint) {
            self.fill_none();
        } else {
            *self = Self::new_of_type(type_hint, self.len());
        }
    }

    /// Returns a reference to the signal at the given index.
    #[inline]
    pub fn get(&self, index: usize) -> Option<AnySignalOpt> {
        match self {
            Self::Float(buffer) => buffer
                .get(index)
                .copied()
                .map(|v| AnySignalOpt::Float(v.into_signal())),
            Self::Int(buffer) => buffer.get(index).copied().map(AnySignalOpt::Int),
            Self::Bool(buffer) => buffer.get(index).copied().map(AnySignalOpt::Bool),
            Self::Midi(buffer) => buffer.get(index).copied().map(AnySignalOpt::Midi),
        }
    }

    /// Returns a mutable reference to the signal at the given index.
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<AnySignalOptMut> {
        match self {
            Self::Float(buffer) => buffer.get_mut(index).map(AnySignalOptMut::Float),
            Self::Int(buffer) => buffer.get_mut(index).map(AnySignalOptMut::Int),
            Self::Bool(buffer) => buffer.get_mut(index).map(AnySignalOptMut::Bool),
            Self::Midi(buffer) => buffer.get_mut(index).map(AnySignalOptMut::Midi),
        }
    }

    /// Returns the signal at the given index.
    #[inline]
    pub fn get_as<S: Signal>(&self, index: usize) -> Option<S> {
        S::try_convert_buffer(self)?
            .get(index)?
            .map(|v| S::from_repr(v))
    }

    /// Returns a mutable reference to the signal at the given index.
    #[inline]
    pub fn get_mut_as<S: Signal>(&mut self, index: usize) -> Option<&mut Option<S::Repr>> {
        S::try_convert_buffer_mut(self)?.get_mut(index)
    }

    /// Sets the signal at the given index.
    ///
    /// # Panics
    ///
    /// Panics if the signal type does not match the buffer type.
    #[inline]
    pub fn set(&mut self, index: usize, value: AnySignalOpt) {
        match (self, value) {
            (Self::Float(buffer), AnySignalOpt::Float(value)) => buffer[index] = value.into_repr(),
            (Self::Int(buffer), AnySignalOpt::Int(value)) => buffer[index] = value.into_repr(),
            (Self::Bool(buffer), AnySignalOpt::Bool(value)) => buffer[index] = value.into_repr(),
            (Self::Midi(buffer), AnySignalOpt::Midi(value)) => buffer[index] = value.into_repr(),
            (this, value) => {
                panic!(
                    "Cannot set signal of different type: {:?} != {:?}",
                    this.signal_type(),
                    value.signal_type()
                );
            }
        }
    }

    /// Clones the given signal and stores it at the given index.
    /// Returns `true` if the signal was set successfully.
    #[cfg_attr(feature = "profiling", inline(never))]
    #[cfg_attr(not(feature = "profiling"), inline)]
    pub fn set_as<S: Signal + Clone>(&mut self, index: usize, value: Option<S>) -> bool {
        if let Some(buf) = S::try_convert_buffer_mut(self) {
            let slot = buf.get_mut(index).unwrap();
            slot.clone_from(&value.into_repr()); // `clone_from` is used to possibly avoid cloning the value twice
            true
        } else {
            false
        }
    }

    /// Sets the signal at the given index to `None`.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    #[inline]
    pub fn set_none(&mut self, index: usize) {
        match self {
            Self::Float(buffer) => buffer[index] = None,
            Self::Int(buffer) => buffer[index] = None,
            Self::Bool(buffer) => buffer[index] = None,
            Self::Midi(buffer) => buffer[index] = None,
        }
    }

    /// Clones the contents of the other buffer into this buffer.
    ///
    /// # Panics
    ///
    /// Panics if the buffer types do not match.
    #[inline]
    pub fn clone_from(&mut self, other: &Self) {
        match (self, other) {
            (Self::Float(this), Self::Float(other)) => {
                this.copy_from_slice(other);
            }
            (Self::Int(this), Self::Int(other)) => {
                this.copy_from_slice(other);
            }
            (Self::Bool(this), Self::Bool(other)) => {
                this.copy_from_slice(other);
            }
            (Self::Midi(this), Self::Midi(other)) => {
                this.copy_from_slice(other);
            }
            _ => panic!("Cannot copy buffer of different type"),
        }
    }

    /// Returns an iterator over the signals in the buffer.
    #[inline]
    pub fn iter(&self) -> SignalBufferIter {
        SignalBufferIter {
            buffer: self,
            index: 0,
        }
    }

    /// Returns a mutable iterator over the signals in the buffer.
    #[inline]
    pub fn iter_mut(&mut self) -> SignalBufferIterMut {
        SignalBufferIterMut {
            buffer: self,
            index: 0,
            _marker: std::marker::PhantomData,
        }
    }
}

/// An iterator over the signals in a buffer.
pub struct SignalBufferIter<'a> {
    buffer: &'a SignalBuffer,
    index: usize,
}

impl Iterator for SignalBufferIter<'_> {
    type Item = AnySignalOpt;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.buffer.len() {
            let signal = match self.buffer {
                SignalBuffer::Float(buffer) => {
                    AnySignalOpt::Float(buffer[self.index].into_signal())
                }
                SignalBuffer::Int(buffer) => AnySignalOpt::Int(buffer[self.index].into_signal()),
                SignalBuffer::Bool(buffer) => AnySignalOpt::Bool(buffer[self.index].into_signal()),
                SignalBuffer::Midi(buffer) => AnySignalOpt::Midi(buffer[self.index].into_signal()),
            };
            self.index += 1;
            Some(signal)
        } else {
            None
        }
    }
}

impl<'a> IntoIterator for &'a SignalBuffer {
    type Item = AnySignalOpt;
    type IntoIter = SignalBufferIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        SignalBufferIter {
            buffer: self,
            index: 0,
        }
    }
}

/// An mutable iterator over the signals in a buffer.
pub struct SignalBufferIterMut<'a> {
    buffer: &'a mut SignalBuffer,
    index: usize,
    _marker: std::marker::PhantomData<AnySignalOptMut<'a>>,
}

impl<'a> Iterator for SignalBufferIterMut<'a> {
    type Item = AnySignalOptMut<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.buffer.len() {
            // SAFETY:
            // We are borrowing the buffer mutably, so we can safely create a mutable reference to the signal.
            // We are also only creating one mutable reference at a time, so there are no issues with aliasing.
            // The lifetime of the mutable reference is limited to the lifetime of the iterator.
            // This is similar to how `std::slice::IterMut` works.
            unsafe {
                let signal = match self.buffer {
                    SignalBuffer::Float(buffer) => AnySignalOptMut::Float(
                        &mut *(&mut buffer[self.index] as *mut OptionRepr<Float>),
                    ),
                    SignalBuffer::Int(buffer) => AnySignalOptMut::Int(
                        &mut *(&mut buffer[self.index] as *mut OptionRepr<i64>),
                    ),
                    SignalBuffer::Bool(buffer) => AnySignalOptMut::Bool(
                        &mut *(&mut buffer[self.index] as *mut OptionRepr<bool>),
                    ),
                    SignalBuffer::Midi(buffer) => AnySignalOptMut::Midi(
                        &mut *(&mut buffer[self.index] as *mut OptionRepr<MidiMessage>),
                    ),
                };
                self.index += 1;
                Some(signal)
            }
        } else {
            None
        }
    }
}

impl<'a> IntoIterator for &'a mut SignalBuffer {
    type Item = AnySignalOptMut<'a>;
    type IntoIter = SignalBufferIterMut<'a>;

    fn into_iter(self) -> Self::IntoIter {
        SignalBufferIterMut {
            buffer: self,
            index: 0,
            _marker: std::marker::PhantomData,
        }
    }
}

impl FromIterator<Float> for SignalBuffer {
    fn from_iter<T: IntoIterator<Item = Float>>(iter: T) -> Self {
        Self::Float(iter.into_iter().collect())
    }
}

impl FromIterator<i64> for SignalBuffer {
    fn from_iter<T: IntoIterator<Item = i64>>(iter: T) -> Self {
        Self::Int(iter.into_iter().collect())
    }
}

impl FromIterator<bool> for SignalBuffer {
    fn from_iter<T: IntoIterator<Item = bool>>(iter: T) -> Self {
        Self::Bool(iter.into_iter().collect())
    }
}

impl FromIterator<MidiMessage> for SignalBuffer {
    fn from_iter<T: IntoIterator<Item = MidiMessage>>(iter: T) -> Self {
        Self::Midi(iter.into_iter().collect())
    }
}
