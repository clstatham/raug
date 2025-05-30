//! Signal types and operations.

use std::{
    any::TypeId,
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use crate::processor::io::SignalSpec;
use type_erased::AnyBuffer;

pub mod type_erased;

/// A type that can be stored in a [buffer](type_erased::AnyBuffer) and processed by a [`Processor`](crate::processor::Processor).
pub trait Signal: Send + Sync + 'static {
    /// The type of the signal.
    #[inline]
    fn signal_type() -> SignalType
    where
        Self: Sized,
    {
        SignalType::of::<Self>()
    }

    /// Creates a new buffer of the given size for this signal type.
    #[inline]
    fn create_buffer(size: usize) -> AnyBuffer
    where
        Self: Sized + Clone + Default,
    {
        AnyBuffer::zeros::<Self>(size)
    }

    /// Creates a [`SignalSpec`](crate::processor::io::SignalSpec) for this signal type with the given name.
    #[inline]
    fn signal_spec(name: impl Into<String>) -> SignalSpec
    where
        Self: Sized,
    {
        SignalSpec::of_type::<Self>(name)
    }
}

impl<T: Signal> Signal for Option<T> {}
impl Signal for f32 {}
impl Signal for bool {}

pub const LIST_INLINE_SIZE: usize = 16;

#[derive(Debug, PartialEq)]
pub struct List<T: Signal> {
    vec: smallvec::SmallVec<[T; LIST_INLINE_SIZE]>,
}

impl<T: Signal> Default for List<T> {
    #[inline]
    fn default() -> Self {
        Self {
            vec: smallvec::SmallVec::default(),
        }
    }
}

impl<T: Signal + Clone> Clone for List<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            vec: self.vec.clone(),
        }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.vec.clone_from(&source.vec);
    }
}

impl<T: Signal + Clone> List<T> {
    /// Creates a new list with the given capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            vec: smallvec::SmallVec::with_capacity(capacity),
        }
    }

    #[inline]
    pub fn from_slice(slice: &[T]) -> Self {
        Self {
            vec: slice.iter().cloned().collect(),
        }
    }

    #[inline]
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
    type Target = smallvec::SmallVec<[T; LIST_INLINE_SIZE]>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.vec
    }
}

impl<T: Signal> DerefMut for List<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vec
    }
}

impl<T: Signal> From<Vec<T>> for List<T> {
    #[inline]
    fn from(vec: Vec<T>) -> Self {
        Self {
            vec: smallvec::SmallVec::from_iter(vec),
        }
    }
}

impl<T: Signal + Clone> From<List<T>> for Vec<T> {
    #[inline]
    fn from(list: List<T>) -> Self {
        list.to_alloc_vec()
    }
}

impl<T: Signal> Signal for List<T> {}

pub const STRING_INLINE_SIZE: usize = LIST_INLINE_SIZE * size_of::<f32>();

#[derive(Debug, PartialEq)]
pub struct Str {
    string: smallstr::SmallString<[u8; STRING_INLINE_SIZE]>,
}

impl Default for Str {
    #[inline]
    fn default() -> Self {
        Self {
            string: smallstr::SmallString::new(),
        }
    }
}

impl Clone for Str {
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

impl Str {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        self.string.as_str()
    }
}

impl Deref for Str {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl From<&str> for Str {
    #[inline]
    fn from(s: &str) -> Self {
        Self {
            string: smallstr::SmallString::from_str(s),
        }
    }
}

impl Signal for Str {}

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

impl From<SignalType> for raug_graph::TypeInfo {
    #[inline]
    fn from(signal_type: SignalType) -> Self {
        Self {
            type_name: signal_type.name,
            type_id: signal_type.id,
        }
    }
}

impl From<raug_graph::TypeInfo> for SignalType {
    #[inline]
    fn from(type_info: raug_graph::TypeInfo) -> Self {
        Self {
            name: type_info.type_name,
            id: type_info.type_id,
        }
    }
}
