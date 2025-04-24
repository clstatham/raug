//! Signal types and operations.

use std::{
    any::TypeId,
    fmt::Debug,
    ops::{Deref, DerefMut},
};

pub mod buffer;
pub mod type_erased;

/// A type that can be stored in a [`Buffer`](buffer::Buffer) and processed by a [`Processor`](crate::processor::Processor).
pub trait Signal: Sized + Clone + Default + Send + Sync + 'static {
    /// The type of the signal.
    #[inline]
    fn signal_type() -> SignalType {
        SignalType::of::<Self>()
    }

    fn as_bool(&self) -> bool;
}

impl<T: Signal> Signal for Option<T> {
    #[inline]
    fn as_bool(&self) -> bool {
        self.is_some()
    }
}
impl<T: Signal> Signal for &'static [T] {
    #[inline]
    fn as_bool(&self) -> bool {
        !self.is_empty()
    }
}
impl Signal for f32 {
    #[inline]
    fn as_bool(&self) -> bool {
        *self > 0.0
    }
}
impl Signal for bool {
    #[inline]
    fn as_bool(&self) -> bool {
        *self
    }
}

#[derive(Debug, PartialEq)]
pub struct List<T: Signal> {
    vec: smallvec::SmallVec<[T; 16]>,
}

impl<T: Signal> Default for List<T> {
    fn default() -> Self {
        Self {
            vec: smallvec::SmallVec::default(),
        }
    }
}

impl<T: Signal> Clone for List<T> {
    fn clone(&self) -> Self {
        Self {
            vec: self.vec.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.vec.clone_from(&source.vec);
    }
}

impl<T: Signal> List<T> {
    /// Creates a new list with the given capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            vec: smallvec::SmallVec::with_capacity(capacity),
        }
    }

    pub fn from_slice(slice: &[T]) -> Self {
        Self {
            vec: slice.iter().cloned().collect(),
        }
    }

    pub fn to_alloc_vec(&self) -> Vec<T> {
        self.vec.to_vec()
    }
}

impl<T: Signal> AsRef<[T]> for List<T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.vec.as_slice()
    }
}

impl<T: Signal> Deref for List<T> {
    type Target = smallvec::SmallVec<[T; 16]>;

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
            vec: smallvec::SmallVec::from_iter(vec),
        }
    }
}

impl<T: Signal> From<List<T>> for Vec<T> {
    fn from(list: List<T>) -> Self {
        list.to_alloc_vec()
    }
}

impl<T: Signal> Signal for List<T> {
    #[inline]
    fn as_bool(&self) -> bool {
        !self.is_empty()
    }
}

#[derive(Debug, PartialEq)]
pub struct StringSignal {
    string: smallstr::SmallString<[u8; 128]>,
}

impl Default for StringSignal {
    fn default() -> Self {
        Self {
            string: smallstr::SmallString::new(),
        }
    }
}

impl Clone for StringSignal {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            string: self.string.clone(),
        }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.string.clone_from(&source.string);
    }
}

impl StringSignal {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        self.string.as_str()
    }
}

impl Deref for StringSignal {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl From<&str> for StringSignal {
    #[inline]
    fn from(s: &str) -> Self {
        Self {
            string: smallstr::SmallString::from_str(s),
        }
    }
}

impl Signal for StringSignal {
    fn as_bool(&self) -> bool {
        !self.is_empty()
    }
}

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
