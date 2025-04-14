//! Signal types and operations.

use std::{
    any::TypeId,
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use allocator::SignalAlloc;
use allocator_api2::SliceExt;

pub mod allocator;
pub mod buffer;
pub mod type_erased;

/// A type that can be stored in a [`Buffer`](buffer::Buffer) and processed by a [`Processor`](crate::processor::Processor).
pub trait Signal: Clone + Default + Debug + Send + Sync + PartialEq + 'static {
    /// The type of the signal.
    #[inline]
    fn signal_type() -> SignalType {
        SignalType::of::<Self>()
    }
}

impl<T: Signal> Signal for Option<T> {}
impl<T: Signal> Signal for &'static [T] {}
impl Signal for f32 {}
impl Signal for f64 {}
impl Signal for i32 {}
impl Signal for i64 {}
impl Signal for bool {}
impl Signal for u32 {}
impl Signal for u64 {}
impl Signal for usize {}

#[derive(Debug, PartialEq)]
pub struct List<T: Signal> {
    vec: allocator_api2::vec::Vec<T, SignalAlloc>,
}

impl<T: Signal> Clone for List<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            vec: SliceExt::to_vec_in(self.vec.as_slice(), SignalAlloc),
        }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.vec.clear();
        self.vec.extend_from_slice(source.vec.as_slice());
    }
}

impl<T: Signal> Default for List<T> {
    fn default() -> Self {
        Self {
            vec: allocator_api2::vec::Vec::new_in(SignalAlloc),
        }
    }
}

impl<T: Signal> List<T> {
    /// Creates a new list with the given capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            vec: allocator_api2::vec::Vec::with_capacity_in(capacity, SignalAlloc),
        }
    }

    pub fn from_slice(slice: &[T]) -> Self {
        Self {
            vec: SliceExt::to_vec_in(slice, SignalAlloc),
        }
    }

    pub fn to_alloc_vec(&self) -> Vec<T> {
        self.vec.as_slice().to_vec()
    }
}

impl<T: Signal> AsRef<[T]> for List<T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.vec.as_slice()
    }
}

impl<T: Signal> Deref for List<T> {
    type Target = allocator_api2::vec::Vec<T, SignalAlloc>;

    fn deref(&self) -> &Self::Target {
        &self.vec
    }
}

impl<T: Signal> DerefMut for List<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vec
    }
}

impl<T: Signal> From<Vec<T>> for List<T> {
    fn from(vec: Vec<T>) -> Self {
        Self {
            vec: SliceExt::to_vec_in(vec.as_slice(), SignalAlloc),
        }
    }
}

impl<T: Signal> From<List<T>> for Vec<T> {
    fn from(list: List<T>) -> Self {
        list.to_alloc_vec()
    }
}

impl<T: Signal> Signal for List<T> {}

#[derive(Debug, PartialEq)]
pub struct StringSignal {
    vec: allocator_api2::vec::Vec<u8, SignalAlloc>,
}

impl Clone for StringSignal {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            vec: SliceExt::to_vec_in(self.vec.as_slice(), SignalAlloc),
        }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.vec.clear();
        self.vec.extend_from_slice(source.vec.as_slice());
    }
}

impl Default for StringSignal {
    fn default() -> Self {
        Self {
            vec: allocator_api2::vec::Vec::new_in(SignalAlloc),
        }
    }
}

impl StringSignal {
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(self.vec.as_slice()).unwrap()
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        Self {
            vec: SliceExt::to_vec_in(s.as_bytes(), SignalAlloc),
        }
    }
}

impl Deref for StringSignal {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl From<&str> for StringSignal {
    fn from(s: &str) -> Self {
        Self::from_str(s)
    }
}

impl Signal for StringSignal {}

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
