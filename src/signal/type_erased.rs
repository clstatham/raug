use std::any::TypeId;

use any_vec::{
    AnyVec,
    any_value::{AnyValue, AnyValueWrapper},
    element::{ElementMut, ElementRef},
    traits::Cloneable,
};

use super::{Signal, SignalType};

#[derive(Clone)]
pub struct ErasedBuffer {
    signal_type: SignalType,
    buf: AnyVec<dyn Send + Sync + Cloneable>,
}

impl ErasedBuffer {
    pub fn empty<T: Signal>() -> Self {
        Self {
            signal_type: SignalType::of::<T>(),
            buf: AnyVec::new::<T>(),
        }
    }

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

    #[inline]
    pub fn as_slice<T: Signal>(&self) -> &[T] {
        assert_eq!(self.signal_type, SignalType::of::<T>());
        unsafe { self.buf.downcast_ref_unchecked::<T>().as_slice() }
    }

    #[inline]
    pub fn as_mut_slice<T: Signal>(&mut self) -> &mut [T] {
        assert_eq!(self.signal_type, SignalType::of::<T>());
        unsafe { self.buf.downcast_mut_unchecked::<T>().as_mut_slice() }
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<ErasedSignalRef> {
        let signal = self.buf.get(index)?;
        Some(ErasedSignalRef { signal })
    }

    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<ErasedSignalMut> {
        let signal = self.buf.get_mut(index)?;
        Some(ErasedSignalMut { signal })
    }

    #[inline]
    pub fn get_as<T: Signal>(&self, index: usize) -> Option<T> {
        assert_eq!(self.signal_type, SignalType::of::<T>());
        Some(*self.get(index)?.as_ref::<T>())
    }

    #[inline]
    pub fn get_mut_as<T: Signal>(&mut self, index: usize) -> Option<&mut T> {
        assert_eq!(self.signal_type, SignalType::of::<T>());
        Some(self.get_mut(index)?.as_mut::<T>())
    }

    #[inline]
    pub fn set_as<T: Signal>(&mut self, index: usize, value: T) -> T {
        assert_eq!(self.signal_type, SignalType::of::<T>());
        let item = self.get_mut_as::<T>(index).expect("Index out of bounds");
        let old = *item;
        *item = value;
        old
    }

    #[inline]
    pub fn signal_type(&self) -> SignalType {
        self.signal_type
    }
}

#[derive(Clone)]
#[repr(transparent)]
pub struct ErasedSignalRef<'a> {
    pub(crate) signal: ElementRef<'a, dyn Send + Sync + Cloneable>,
}

impl<'a> ErasedSignalRef<'a> {
    #[allow(clippy::should_implement_trait)]
    #[inline]
    pub fn as_ref<T: Signal>(&self) -> &'a T {
        assert!(self.signal.value_typeid() == TypeId::of::<T>());
        unsafe { self.signal.downcast_ref_unchecked::<T>() }
    }

    #[inline]
    pub fn value_type_id(&self) -> TypeId {
        self.signal.value_typeid()
    }
}

#[repr(transparent)]
pub struct ErasedSignalMut<'a> {
    pub(crate) signal: ElementMut<'a, dyn Send + Sync + Cloneable>,
}

impl<'a> ErasedSignalMut<'a> {
    #[allow(clippy::should_implement_trait)]
    #[inline]
    pub fn as_ref<T: Signal>(&self) -> &'a T {
        assert!(self.signal.value_typeid() == TypeId::of::<T>());
        unsafe { self.signal.downcast_ref_unchecked::<T>() }
    }

    #[allow(clippy::should_implement_trait)]
    #[inline]
    pub fn as_mut<T: Signal>(&mut self) -> &'a mut T {
        assert!(self.signal.value_typeid() == TypeId::of::<T>());
        unsafe { self.signal.downcast_mut_unchecked::<T>() }
    }

    #[inline]
    pub fn value_type_id(&self) -> TypeId {
        self.signal.value_typeid()
    }
}
