//! Signal types and operations.

use std::{any::TypeId, fmt::Debug};

use repr::{NicheFloat, Repr};

pub mod buffer;
pub mod repr;
pub mod type_erased;

/// A type that can be stored in a [`Buffer`] and processed by a [`Processor`](crate::processor::Processor).
pub trait Signal:
    Copy + Default + Debug + Send + Sync + PartialEq + From<Self::Repr> + 'static
{
    type Repr: Repr<Self> + Copy + Debug + Send + Sync + PartialEq + 'static;

    #[inline]
    fn from_repr(repr: Self::Repr) -> Self {
        repr.into()
    }

    #[inline]
    fn into_repr(self) -> Self::Repr {
        self.into()
    }

    /// The type of the signal.
    #[inline]
    fn signal_type() -> SignalType {
        SignalType::of::<Self>()
    }

    fn cast<T: Signal + From<Self>>(self) -> T {
        T::from(self)
    }
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
    fn set(&mut self, value: T) -> Option<T>;
}

impl<T: Signal> OptRepr<T> for Option<T::Repr> {
    #[inline]
    fn into_signal(self) -> Option<T> {
        self.map(|v| T::from_repr(v))
    }

    #[inline]
    fn set(&mut self, value: T) -> Option<T> {
        let old = self.into_signal();
        *self = Some(value.into_repr());
        old
    }
}

impl Signal for f32 {
    type Repr = NicheFloat;
}

impl Signal for i64 {
    type Repr = i64;
}

impl Signal for bool {
    type Repr = bool;
}

/// A signal type.
#[derive(Debug, Clone, Copy)]
pub struct SignalType {
    name: &'static str,
    id: TypeId,
}

impl SignalType {
    /// Gets the signal type for the given signal.
    #[inline]
    pub fn of<T: Signal>() -> Self {
        Self {
            name: std::any::type_name::<T>(),
            id: TypeId::of::<T>(),
        }
    }

    /// Returns the signal type name.
    #[inline]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// Returns the signal type ID.
    #[inline]
    pub const fn id(&self) -> TypeId {
        self.id
    }

    /// Returns `true` if the signal type is the same as the given type.
    #[inline]
    pub fn is<T: Signal>(&self) -> bool {
        self.id == TypeId::of::<T>()
    }
}

impl PartialEq for SignalType {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for SignalType {}
