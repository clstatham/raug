//! Signal types and operations.

use std::fmt::Debug;

use buffer::{Buffer, SignalBuffer};
use repr::{FloatRepr, Repr};

pub mod buffer;
pub mod repr;

mod sealed {
    pub trait Sealed {}
    impl Sealed for f32 {}
    impl Sealed for f64 {}
    impl Sealed for i64 {}
    impl Sealed for bool {}
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

impl Signal for f32 {
    type Repr = FloatRepr;

    #[inline]
    fn from_repr(repr: Self::Repr) -> Self {
        repr.into_signal()
    }

    #[inline]
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

    #[inline]
    fn from_repr(repr: Self::Repr) -> Self {
        repr
    }

    #[inline]
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

    #[inline]
    fn from_repr(repr: Self::Repr) -> Self {
        repr
    }

    #[inline]
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

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AnySignal {
    Float(f32),
    Int(i64),
    Bool(bool),
}

impl AnySignal {
    /// Returns the type of the signal.
    #[inline]
    pub fn signal_type(&self) -> SignalType {
        match self {
            Self::Float(_) => SignalType::Float,
            Self::Int(_) => SignalType::Int,
            Self::Bool(_) => SignalType::Bool,
        }
    }

    #[inline]
    pub fn default_of_type(signal_type: &SignalType) -> Self {
        match signal_type {
            SignalType::Float => AnySignal::Float(0.0),
            SignalType::Int => AnySignal::Int(0),
            SignalType::Bool => AnySignal::Bool(false),
        }
    }

    /// Attempts to cast the signal to the given signal type.
    ///
    /// Currently, the following conversions are supported:
    ///
    /// | From \ To | f32 | Int | Bool | Midi |
    /// |-----------|-------|-----|------|------|
    /// | f32     | -     | Yes | Yes  | -    |
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
            (Self::Int(int), SignalType::Float) => Some(Self::Float(*int as f32)),
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
    Float(Option<f32>),

    /// An integer.
    Int(Option<i64>),

    /// A boolean.
    Bool(Option<bool>),
}

impl AnySignalOpt {
    /// Creates a new signal of the given type with no value.
    pub fn default_of_type(signal_type: &SignalType) -> Self {
        match signal_type {
            SignalType::Float => AnySignalOpt::Float(None),
            SignalType::Int => AnySignalOpt::Int(None),
            SignalType::Bool => AnySignalOpt::Bool(None),
        }
    }

    /// Returns `true` if the signal is `Some`.
    pub fn is_some(&self) -> bool {
        match self {
            Self::Float(float) => float.is_some(),
            Self::Int(int) => int.is_some(),
            Self::Bool(bool) => bool.is_some(),
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
        )
    }

    /// Returns the type of the signal.
    #[inline]
    pub fn signal_type(&self) -> SignalType {
        match self {
            Self::Float(_) => SignalType::Float,
            Self::Int(_) => SignalType::Int,
            Self::Bool(_) => SignalType::Bool,
        }
    }

    #[inline]
    pub fn as_type<T: Signal>(&self) -> Option<T> {
        self.try_into_any_signal().and_then(T::try_from_any_signal)
    }

    /// Attempts to cast the signal to the given signal type.
    #[inline]
    pub fn cast(&self, target: SignalType) -> Option<Self> {
        if self.signal_type() == target {
            return Some(*self);
        }
        match (self, target) {
            (Self::Float(float), SignalType::Int) => float.map(|f| Self::Int(Some(f as i64))),
            (Self::Float(float), SignalType::Bool) => float.map(|f| Self::Bool(Some(f != 0.0))),
            (Self::Int(int), SignalType::Float) => int.map(|i| Self::Float(Some(i as f32))),
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
        }
    }

    #[inline]
    pub fn set_none(&mut self) {
        match self {
            Self::Float(float) => *float = None,
            Self::Int(int) => *int = None,
            Self::Bool(bool) => *bool = None,
        }
    }
}

pub enum AnySignalOptMut<'a> {
    Float(&'a mut OptionRepr<f32>),
    Int(&'a mut OptionRepr<i64>),
    Bool(&'a mut OptionRepr<bool>),
}

impl AnySignalOptMut<'_> {
    /// Returns the type of the signal.
    #[inline]
    pub fn signal_type(&self) -> SignalType {
        match self {
            Self::Float(_) => SignalType::Float,
            Self::Int(_) => SignalType::Int,
            Self::Bool(_) => SignalType::Bool,
        }
    }

    /// Returns `true` if the signal is `Some`.
    #[inline]
    pub fn is_some(&self) -> bool {
        match self {
            Self::Float(float) => float.is_some(),
            Self::Int(int) => int.is_some(),
            Self::Bool(bool) => bool.is_some(),
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
            _ => panic!("Cannot set signal of different type"),
        }
    }

    #[inline]
    pub fn set_none(&mut self) {
        match self {
            Self::Float(float) => **float = None,
            Self::Int(int) => **int = None,
            Self::Bool(bool) => **bool = None,
        }
    }
}

/// A signal type.
#[derive(Debug, Clone, PartialEq, Eq, Copy, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SignalType {
    /// A floating-point signal.
    Float,

    /// An integer signal.
    Int,

    /// A boolean signal.
    Bool,
}
