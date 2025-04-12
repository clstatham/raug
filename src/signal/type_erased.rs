use std::any::TypeId;

use any_vec::{
    AnyVec,
    any_value::{AnyValue, AnyValueWrapper},
    element::{ElementMut, ElementRef},
    traits::Cloneable,
};

use crate::signal::OptRepr;

use super::{OptionRepr, Signal, buffer::Buffer};

#[derive(Clone)]
#[repr(transparent)]
pub struct ErasedBuffer {
    pub(crate) buf: AnyVec<dyn Send + Sync + Cloneable>,
}

impl ErasedBuffer {
    pub fn empty<T: Signal>() -> Self {
        Self {
            buf: AnyVec::new::<OptionRepr<T>>(),
        }
    }

    pub fn from_buffer<T: Signal>(mut buffer: Buffer<T>) -> Self {
        let mut buf = AnyVec::new::<OptionRepr<T>>();
        buf.reserve(buffer.len());
        for item in buffer.drain(..) {
            buf.push(AnyValueWrapper::new(item));
        }

        Self { buf }
    }

    pub fn drain_into_buffer<T: Signal>(&mut self, buffer: &mut Buffer<T>) {
        assert_eq!(self.buf.element_typeid(), TypeId::of::<OptionRepr<T>>());
        assert!(self.buf.len() <= buffer.len());
        for (i, item) in self.buf.drain(..).enumerate() {
            buffer[i] = item.downcast::<OptionRepr<T>>().unwrap();
        }
    }

    pub fn into_buffer<T: Signal>(mut self) -> Buffer<T> {
        assert_eq!(self.buf.element_typeid(), TypeId::of::<OptionRepr<T>>());
        let mut buffer = Buffer::zeros(self.buf.len());
        self.drain_into_buffer(&mut buffer);

        buffer
    }

    #[inline]
    pub fn as_slice<T: Signal>(&self) -> &[OptionRepr<T>] {
        assert_eq!(self.buf.element_typeid(), TypeId::of::<OptionRepr<T>>());
        unsafe {
            self.buf
                .downcast_ref_unchecked::<OptionRepr<T>>()
                .as_slice()
        }
    }

    #[inline]
    pub fn as_mut_slice<T: Signal>(&mut self) -> &mut [OptionRepr<T>] {
        assert_eq!(self.buf.element_typeid(), TypeId::of::<OptionRepr<T>>());
        self.buf
            .downcast_mut::<OptionRepr<T>>()
            .unwrap()
            .as_mut_slice()
    }

    pub fn resize<T: Signal>(&mut self, len: usize) {
        assert_eq!(self.buf.element_typeid(), TypeId::of::<OptionRepr<T>>());
        let diff = len as isize - self.buf.len() as isize;
        if diff > 0 {
            self.buf.reserve(diff as usize);
            for _ in 0..diff {
                self.buf.push(AnyValueWrapper::<OptionRepr<T>>::new(None));
            }
        } else {
            for _ in 0..-diff {
                self.buf.pop();
            }
        }
    }

    pub fn fill<T: Signal>(&mut self, value: impl Into<OptionRepr<T>>) {
        self.as_mut_slice::<T>().fill(value.into());
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
        assert_eq!(self.buf.element_typeid(), TypeId::of::<OptionRepr<T>>());
        self.get(index)?.as_ref::<T>().into_signal()
    }

    #[inline]
    pub fn get_mut_as<T: Signal>(&mut self, index: usize) -> Option<&mut OptionRepr<T>> {
        assert_eq!(self.buf.element_typeid(), TypeId::of::<OptionRepr<T>>());
        Some(self.get_mut(index)?.as_mut::<T>())
    }

    #[inline]
    pub fn set_as<T: Signal>(&mut self, index: usize, value: T) -> Option<T> {
        assert_eq!(self.buf.element_typeid(), TypeId::of::<OptionRepr<T>>());
        let item = self.get_mut_as::<T>(index).expect("Index out of bounds");
        item.set(value)
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
    pub fn as_ref<T: Signal>(&self) -> &'a OptionRepr<T> {
        assert!(self.signal.value_typeid() == TypeId::of::<OptionRepr<T>>());
        unsafe { self.signal.downcast_ref_unchecked::<OptionRepr<T>>() }
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
    pub fn as_ref<T: Signal>(&self) -> &'a OptionRepr<T> {
        assert!(self.signal.value_typeid() == TypeId::of::<OptionRepr<T>>());
        unsafe { self.signal.downcast_ref_unchecked::<OptionRepr<T>>() }
    }

    #[allow(clippy::should_implement_trait)]
    #[inline]
    pub fn as_mut<T: Signal>(&mut self) -> &'a mut OptionRepr<T> {
        assert!(self.signal.value_typeid() == TypeId::of::<OptionRepr<T>>());
        unsafe { self.signal.downcast_mut_unchecked::<OptionRepr<T>>() }
    }

    #[inline]
    pub fn value_type_id(&self) -> TypeId {
        self.signal.value_typeid()
    }
}
