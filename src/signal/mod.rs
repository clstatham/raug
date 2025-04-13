//! Signal types and operations.

use std::{any::TypeId, fmt::Debug};

pub mod buffer;
pub mod type_erased;

/// A type that can be stored in a [`Buffer`](buffer::Buffer) and processed by a [`Processor`](crate::processor::Processor).
pub trait Signal: Copy + Default + Debug + Send + Sync + PartialEq + 'static {
    /// The type of the signal.
    #[inline]
    fn signal_type() -> SignalType {
        SignalType::of::<Self>()
    }
}

impl<T: Signal> Signal for Option<T> {}
impl Signal for f32 {}
impl Signal for f64 {}
impl Signal for i32 {}
impl Signal for i64 {}
impl Signal for bool {}
impl Signal for u32 {}
impl Signal for u64 {}
impl Signal for usize {}

/// Type information for a signal.
#[derive(Clone, Copy)]
pub struct SignalType {
    /// The name of the signal type.
    name: &'static str,

    /// The type ID of the signal.
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

impl std::fmt::Debug for SignalType {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SignalType({})", self.name)?;
        Ok(())
    }
}
