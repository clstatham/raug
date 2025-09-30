//! Type-erased buffer for signals.

use any_vec::{
    AnyVec,
    any_value::{AnyValueCloneable, AnyValueWrapper},
    element::{ElementMut, ElementRef},
    traits::Cloneable,
};

use super::{Signal, SignalType};

/// A type-erased buffer that can hold signals of any type.
pub struct AnyBuffer {
    element_signal_type: SignalType,
    buf: AnyVec<dyn Send + Sync + Cloneable>,
}

impl AnyBuffer {
    /// Creates a new empty buffer of the given type.
    pub fn new<T: Signal + Clone>() -> Self {
        Self {
            element_signal_type: T::signal_type(),
            buf: AnyVec::new::<T>(),
        }
    }

    /// Creates a new buffer of the given type with the specified length, initialized to the default value for `T`.
    pub fn zeros<T: Signal + Clone + Default>(len: usize) -> Self {
        let mut buf = AnyVec::with_capacity::<T>(len);
        for _ in 0..len {
            buf.push(AnyValueWrapper::<T>::new(T::default()));
        }

        Self {
            element_signal_type: T::signal_type(),
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
    #[inline]
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    /// Returns `true` if the buffer is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    /// Returns a reference to the signal at the given index, if the type matches.
    ///
    /// Returns `None` if the index is out of bounds.
    #[inline]
    pub fn get(&'_ self, index: usize) -> Option<AnySignalRef<'_>> {
        let signal = self.buf.get(index)?;
        Some(AnySignalRef {
            signal_type: self.element_signal_type,
            signal,
        })
    }

    /// Returns a mutable reference to the signal at the given index, if the type matches.
    ///
    /// Returns `None` if the index is out of bounds.
    #[inline]
    pub fn get_mut(&'_ mut self, index: usize) -> Option<AnySignalMut<'_>> {
        let signal = self.buf.get_mut(index)?;
        Some(AnySignalMut {
            signal_type: self.element_signal_type,
            signal,
        })
    }

    /// Returns a reference to the signal at the given index, if the type matches.
    #[inline]
    pub fn get_as<T: Signal>(&self, index: usize) -> Option<&T> {
        self.get(index)?.downcast_ref::<T>()
    }

    /// Returns a mutable reference to the signal at the given index, if the type matches.
    #[inline]
    pub fn get_mut_as<T: Signal>(&mut self, index: usize) -> Option<&mut T> {
        self.get_mut(index)?.downcast_mut::<T>()
    }

    #[inline]
    pub fn clone_from(&mut self, other: &AnyBuffer) {
        assert_eq!(self.element_signal_type, other.element_signal_type);
        self.buf.clear();
        self.buf.reserve(other.len());
        for i in 0..other.len() {
            let signal = other.get(i).unwrap();
            let signal = signal.signal.lazy_clone();
            self.buf.push(signal);
        }
    }

    /// Returns the [`SignalType`] of the signals contained in this buffer.
    #[inline]
    pub fn element_signal_type(&self) -> SignalType {
        self.element_signal_type
    }
}

impl<T: Signal> AsRef<[T]> for AnyBuffer {
    #[inline]
    #[track_caller]
    fn as_ref(&self) -> &[T] {
        self.as_slice().expect("Signal type mismatch")
    }
}

/// A reference to a signal of any type.
#[derive(Clone)]
pub struct AnySignalRef<'a> {
    signal_type: SignalType,
    signal: ElementRef<'a, dyn Send + Sync + Cloneable>,
}

impl<'a> AnySignalRef<'a> {
    /// Returns a reference to the signal, if the type matches.
    ///
    /// # Panics
    ///
    /// Panics if the type of the signal does not match the expected type.
    #[allow(clippy::should_implement_trait)]
    #[inline]
    pub fn downcast_ref<T: Signal>(&self) -> Option<&'a T> {
        self.signal.downcast_ref::<T>()
    }

    /// Returns the [`SignalType`] of the signal.
    #[inline]
    pub fn signal_type(&self) -> SignalType {
        self.signal_type
    }
}

/// A mutable reference to a signal of any type.
pub struct AnySignalMut<'a> {
    signal_type: SignalType,
    signal: ElementMut<'a, dyn Send + Sync + Cloneable>,
}

impl<'a> AnySignalMut<'a> {
    /// Returns a reference to the signal, if the type matches.
    #[allow(clippy::should_implement_trait)]
    #[inline]
    pub fn downcast_ref<T: Signal>(&self) -> Option<&'a T> {
        self.signal.downcast_ref::<T>()
    }

    /// Returns a mutable reference to the signal, if the type matches.
    #[allow(clippy::should_implement_trait)]
    #[inline]
    pub fn downcast_mut<T: Signal>(&mut self) -> Option<&'a mut T> {
        self.signal.downcast_mut::<T>()
    }

    /// Returns the [`SignalType`] of the signal.
    #[inline]
    pub fn signal_type(&self) -> SignalType {
        self.signal_type
    }

    #[inline]
    pub fn clone_from(&mut self, other: &AnySignalRef) {
        self.signal
            .lazy_clone()
            .clone_from(&other.signal.lazy_clone())
    }
}

impl<T: Signal> AsRef<T> for AnySignalRef<'_> {
    fn as_ref(&self) -> &T {
        self.downcast_ref::<T>()
            .unwrap_or_else(|| panic!("Failed to downcast AnySignalRef to {:?}", T::signal_type()))
    }
}

impl<T: Signal> AsRef<T> for AnySignalMut<'_> {
    fn as_ref(&self) -> &T {
        self.downcast_ref::<T>()
            .unwrap_or_else(|| panic!("Failed to downcast AnySignalMut to {:?}", T::signal_type()))
    }
}

impl<T: Signal> AsMut<T> for AnySignalMut<'_> {
    fn as_mut(&mut self) -> &mut T {
        self.downcast_mut::<T>()
            .unwrap_or_else(|| panic!("Failed to downcast AnySignalMut to {:?}", T::signal_type()))
    }
}
