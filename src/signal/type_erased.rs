//! Type-erased buffer for signals.

use any_vec::{
    AnyVec,
    any_value::AnyValueWrapper,
    element::{ElementMut, ElementRef},
    traits::Cloneable,
};

use super::{Signal, SignalType};

/// A type-erased buffer that can hold signals of any type.
#[derive(Clone)]
pub struct ErasedBuffer {
    signal_type: SignalType,
    buf: AnyVec<dyn Send + Sync + Cloneable>,
}

impl ErasedBuffer {
    /// Creates a new empty buffer of the given type.
    pub fn empty<T: Signal>() -> Self {
        Self {
            signal_type: SignalType::of::<T>(),
            buf: AnyVec::new::<T>(),
        }
    }

    /// Creates a new buffer of the given type with the specified length, initialized to the default value for `T`.
    pub fn zeros<T: Signal>(len: usize) -> Self {
        let mut buf = AnyVec::new::<T>();
        buf.reserve(len);
        for _ in 0..len {
            buf.push(AnyValueWrapper::<T>::new(T::default()));
        }

        Self {
            signal_type: SignalType::of::<T>(),
            buf,
        }
    }

    /// Returns a view of the buffer as a slice of the given type, if the type matches.
    #[inline]
    pub fn as_slice<T: Signal>(&self) -> Option<&[T]> {
        Some(self.buf.downcast_ref()?.as_slice())
    }

    /// Returns a view of the buffer as a mutable slice of the given type, if the type matches.
    #[inline]
    pub fn as_mut_slice<T: Signal>(&mut self) -> Option<&mut [T]> {
        Some(self.buf.downcast_mut()?.as_mut_slice())
    }

    /// Returns the length of the buffer.
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    /// Returns `true` if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    /// Returns a reference to the signal at the given index, if the type matches.
    ///
    /// Returns `None` if the index is out of bounds.
    #[inline]
    pub fn get(&self, index: usize) -> Option<ErasedSignalRef> {
        let signal = self.buf.get(index)?;
        Some(ErasedSignalRef {
            signal_type: self.signal_type,
            signal,
        })
    }

    /// Returns a mutable reference to the signal at the given index, if the type matches.
    ///
    /// Returns `None` if the index is out of bounds.
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<ErasedSignalMut> {
        let signal = self.buf.get_mut(index)?;
        Some(ErasedSignalMut {
            signal_type: self.signal_type,
            signal,
        })
    }

    /// Returns a copy of the signal at the given index, if the type matches.
    #[inline]
    pub fn get_as<T: Signal>(&self, index: usize) -> Option<T> {
        self.get(index)?.as_ref::<T>().cloned()
    }

    /// Returns a mutable reference to the signal at the given index, if the type matches.
    #[inline]
    pub fn get_mut_as<T: Signal>(&mut self, index: usize) -> Option<&mut T> {
        self.get_mut(index)?.as_mut::<T>()
    }

    /// Returns the [`SignalType`] of the signals contained in this buffer.
    #[inline]
    pub fn signal_type(&self) -> SignalType {
        self.signal_type
    }
}

/// A reference to a signal of any type.
#[derive(Clone)]
pub struct ErasedSignalRef<'a> {
    signal_type: SignalType,
    signal: ElementRef<'a, dyn Send + Sync + Cloneable>,
}

impl<'a> ErasedSignalRef<'a> {
    /// Returns a reference to the signal, if the type matches.
    ///
    /// # Panics
    ///
    /// Panics if the type of the signal does not match the expected type.
    #[allow(clippy::should_implement_trait)]
    #[inline]
    pub fn as_ref<T: Signal>(&self) -> Option<&'a T> {
        self.signal.downcast_ref::<T>()
    }

    /// Returns the [`SignalType`] of the signal.
    #[inline]
    pub fn signal_type(&self) -> SignalType {
        self.signal_type
    }
}

/// A mutable reference to a signal of any type.
pub struct ErasedSignalMut<'a> {
    signal_type: SignalType,
    signal: ElementMut<'a, dyn Send + Sync + Cloneable>,
}

impl<'a> ErasedSignalMut<'a> {
    /// Returns a reference to the signal, if the type matches.
    #[allow(clippy::should_implement_trait)]
    #[inline]
    pub fn as_ref<T: Signal>(&self) -> Option<&'a T> {
        self.signal.downcast_ref::<T>()
    }

    /// Returns a mutable reference to the signal, if the type matches.
    #[allow(clippy::should_implement_trait)]
    #[inline]
    pub fn as_mut<T: Signal>(&mut self) -> Option<&'a mut T> {
        self.signal.downcast_mut::<T>()
    }

    /// Returns the [`SignalType`] of the signal.
    #[inline]
    pub fn signal_type(&self) -> SignalType {
        self.signal_type
    }
}
