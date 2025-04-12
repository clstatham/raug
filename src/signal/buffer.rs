use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
    path::Path,
};

use super::{AnySignalOpt, AnySignalOptMut, OptRepr, OptSignal, OptionRepr, Signal, SignalType};

/// A contiguous buffer of signals.
#[derive(PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Buffer<T: Signal> {
    buf: Vec<Option<T::Repr>>,
}

impl<T: Signal> Debug for Buffer<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.buf.iter()).finish()
    }
}

impl<T: Signal> Buffer<T> {
    /// Creates a new buffer of the given length filled with `None`.
    #[inline]
    pub fn zeros(length: usize) -> Self {
        Buffer {
            buf: vec![None; length],
        }
    }

    /// Clones the slice into a new buffer. All elements are wrapped in `Some`.
    #[inline]
    pub fn from_slice(value: &[T]) -> Self {
        Buffer {
            buf: value.iter().map(|v| Some(v.into_repr())).collect(),
        }
    }

    /// Copies the other buffer into this buffer using a memcpy.
    ///
    /// The inner type must be [`Copy`].
    ///
    /// This is faster than using [`Buffer::from_slice`] for large buffers that are already allocated.
    #[inline]
    pub fn copy_from(&mut self, value: impl AsRef<[Option<T::Repr>]>)
    where
        T: Copy,
    {
        self.buf.copy_from_slice(value.as_ref());
    }
}

impl Buffer<f32> {
    /// Loads a buffer from a WAV file.
    pub fn load_wav(path: impl AsRef<Path>) -> Result<Self, hound::Error> {
        let reader = hound::WavReader::open(path)?;
        if reader.spec().channels == 1 {
            let samples: Result<Vec<_>, hound::Error> = reader
                .into_samples::<f32>()
                .map(|sample| Ok(sample?.into()))
                .collect();
            let samples = samples?;

            Ok(Buffer::from_slice(&samples))
        } else {
            let channels = reader.spec().channels;

            let samples: Result<Vec<_>, hound::Error> = reader
                .into_samples::<f32>()
                .step_by(channels as usize)
                .map(|sample| Ok(sample?.into()))
                .collect();
            let samples = samples?;

            Ok(Buffer::from_slice(&samples))
        }
    }

    /// Saves the buffer to a WAV file. [`None`] entries are written as silence.
    pub fn save_wav(&self, path: impl AsRef<Path>, sample_rate: u32) -> Result<(), hound::Error> {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        let mut writer = hound::WavWriter::create(path, spec)?;
        for sample in self.buf.iter() {
            if let Some(sample) = sample.as_ref() {
                let sample = f32::from_repr(*sample);
                writer.write_sample(sample)?;
            } else {
                writer.write_sample(0.0)?;
            }
        }
        writer.finalize()?;
        Ok(())
    }
}

impl<T: Signal> Deref for Buffer<T> {
    type Target = Vec<Option<T::Repr>>;
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.buf.as_ref()
    }
}

impl<T: Signal> DerefMut for Buffer<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.buf.as_mut()
    }
}

impl<T: Signal> AsRef<Vec<Option<T::Repr>>> for Buffer<T> {
    #[inline]
    fn as_ref(&self) -> &Vec<Option<T::Repr>> {
        self.buf.as_ref()
    }
}

impl<'a, T: Signal> IntoIterator for &'a Buffer<T> {
    type Item = &'a Option<T::Repr>;
    type IntoIter = std::slice::Iter<'a, Option<T::Repr>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.buf.iter()
    }
}

impl<'a, T: Signal> IntoIterator for &'a mut Buffer<T> {
    type Item = &'a mut Option<T::Repr>;
    type IntoIter = std::slice::IterMut<'a, Option<T::Repr>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.buf.iter_mut()
    }
}

impl<T: Signal> FromIterator<T> for Buffer<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Buffer {
            buf: iter.into_iter().map(|v| Some(v.into_repr())).collect(),
        }
    }
}

/// A buffer of signals with a compile-time-known size.
#[derive(PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct FixedBuffer<T: Signal, const N: usize> {
    buf: [Option<T::Repr>; N],
}

impl<T: Signal, const N: usize> Debug for FixedBuffer<T, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.buf.iter()).finish()
    }
}

impl<T: Signal, const N: usize> FixedBuffer<T, N> {
    /// Creates a new buffer of the given length filled with `None`.
    #[inline]
    pub fn zeros() -> Self {
        FixedBuffer { buf: [None; N] }
    }

    /// Clones the slice into a new buffer. All elements are wrapped in `Some`.
    #[inline]
    pub fn from_slice(value: &[T; N]) -> Self {
        let mut buf = [None; N];
        for (i, v) in value.iter().enumerate() {
            buf[i] = Some(v.into_repr());
        }
        FixedBuffer { buf }
    }

    /// Copies the other buffer into this buffer using a memcpy.
    ///
    /// The inner type must be [`Copy`].
    ///
    /// This is faster than using [`Buffer::from_slice`] for large buffers that are already allocated.
    #[inline]
    pub fn copy_from(&mut self, value: &[Option<T::Repr>])
    where
        T: Copy,
    {
        self.buf.copy_from_slice(value);
    }
}

impl<T: Signal, const N: usize> Deref for FixedBuffer<T, N> {
    type Target = [Option<T::Repr>; N];
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.buf
    }
}

impl<T: Signal, const N: usize> DerefMut for FixedBuffer<T, N> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buf
    }
}

impl<T: Signal, const N: usize> AsRef<[Option<T::Repr>; N]> for FixedBuffer<T, N> {
    #[inline]
    fn as_ref(&self) -> &[Option<T::Repr>; N] {
        &self.buf
    }
}

impl<'a, T: Signal, const N: usize> IntoIterator for &'a FixedBuffer<T, N> {
    type Item = &'a Option<T::Repr>;
    type IntoIter = std::slice::Iter<'a, Option<T::Repr>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.buf.iter()
    }
}

impl<'a, T: Signal, const N: usize> IntoIterator for &'a mut FixedBuffer<T, N> {
    type Item = &'a mut Option<T::Repr>;
    type IntoIter = std::slice::IterMut<'a, Option<T::Repr>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.buf.iter_mut()
    }
}

impl<T: Signal, const N: usize> FromIterator<T> for FixedBuffer<T, N> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let iter = iter.into_iter();
        assert_eq!(iter.size_hint().0, N, "Buffer size mismatch");
        let mut buf = [None; N];
        for (i, v) in iter.enumerate() {
            buf[i] = Some(v.into_repr());
        }
        FixedBuffer { buf }
    }
}

/// A buffer of signals that can hold any signal type.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SignalBuffer {
    /// A buffer of floating-point signals.
    Float(Buffer<f32>),

    /// A buffer of integer signals.
    Int(Buffer<i64>),

    /// A buffer of boolean signals.
    Bool(Buffer<bool>),
}

impl SignalBuffer {
    /// Creates a new buffer of the given type with the given length filled with `None`.
    pub fn new_of_type(signal_type: SignalType, length: usize) -> Self {
        match signal_type {
            SignalType::Float => Self::Float(Buffer::zeros(length)),
            SignalType::Int => Self::Int(Buffer::zeros(length)),
            SignalType::Bool => Self::Bool(Buffer::zeros(length)),
        }
    }

    /// Returns the type of the buffer.
    #[inline]
    pub fn signal_type(&self) -> SignalType {
        match self {
            Self::Float(_) => SignalType::Float,
            Self::Int(_) => SignalType::Int,
            Self::Bool(_) => SignalType::Bool,
        }
    }

    /// Returns `true` if the buffer is of the given type.
    #[inline]
    pub fn is_type(&self, signal_type: SignalType) -> bool {
        self.signal_type() == signal_type
    }

    /// Returns a reference to the buffer as a buffer of the given signal type, if it is of that type.
    #[inline]
    pub fn as_type<S: Signal>(&self) -> Option<&Buffer<S>> {
        S::try_convert_buffer(self)
    }

    /// Returns a mutable reference to the buffer as a buffer of the given signal type, if it is of that type.
    #[inline]
    pub fn as_type_mut<S: Signal>(&mut self) -> Option<&mut Buffer<S>> {
        S::try_convert_buffer_mut(self)
    }

    /// Returns the length of the buffer.
    #[inline]
    pub fn len(&self) -> usize {
        match self {
            Self::Float(buffer) => buffer.len(),
            Self::Int(buffer) => buffer.len(),
            Self::Bool(buffer) => buffer.len(),
        }
    }

    /// Returns `true` if the buffer is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Resizes the buffer to the given length, filling the new elements with the given value.
    ///
    /// # Panics
    ///
    /// Panics if the value type does not match the buffer type.
    pub fn resize(&mut self, length: usize, value: impl Into<AnySignalOpt>) {
        let value = value.into();
        match (self, value) {
            (Self::Float(buffer), AnySignalOpt::Float(value)) => {
                buffer.resize(length, value.into_repr())
            }
            (Self::Int(buffer), AnySignalOpt::Int(value)) => {
                buffer.resize(length, value.into_repr())
            }
            (Self::Bool(buffer), AnySignalOpt::Bool(value)) => {
                buffer.resize(length, value.into_repr())
            }
            _ => panic!("Cannot resize buffer with value of different type"),
        }
    }

    /// Fills the buffer with the given value.
    ///
    /// # Panics
    ///
    /// Panics if the value type does not match the buffer type.
    pub fn fill(&mut self, value: impl Into<AnySignalOpt>) {
        let value = value.into();
        match (self, value) {
            (Self::Float(buffer), AnySignalOpt::Float(value)) => buffer.fill(value.into_repr()),
            (Self::Int(buffer), AnySignalOpt::Int(value)) => buffer.fill(value.into_repr()),
            (Self::Bool(buffer), AnySignalOpt::Bool(value)) => buffer.fill(value.into_repr()),
            _ => panic!("Cannot fill buffer with value of different type"),
        }
    }

    /// Resizes the buffer to the given length, filling the new elements with `None`.
    pub fn resize_default(&mut self, length: usize) {
        match self {
            Self::Float(buffer) => buffer.resize(length, None),
            Self::Int(buffer) => buffer.resize(length, None),
            Self::Bool(buffer) => buffer.resize(length, None),
        }
    }

    /// Resizes the buffer based on the given type hint.
    pub fn resize_with_hint(&mut self, length: usize, type_hint: SignalType) {
        let signal_type = self.signal_type();
        if signal_type == type_hint {
            self.resize_default(length);
        } else {
            *self = Self::new_of_type(type_hint, length);
        }
    }

    /// Fills the buffer with `None`.
    pub fn fill_none(&mut self) {
        match self {
            Self::Float(buffer) => buffer.fill(None),
            Self::Int(buffer) => buffer.fill(None),
            Self::Bool(buffer) => buffer.fill(None),
        }
    }

    /// Fills the buffer based on the given type hint.
    pub fn fill_with_hint(&mut self, type_hint: SignalType) {
        let signal_type = self.signal_type();
        if signal_type == type_hint {
            self.fill_none();
        } else {
            *self = Self::new_of_type(type_hint, self.len());
        }
    }

    /// Returns a reference to the signal at the given index.
    #[inline]
    pub fn get(&self, index: usize) -> Option<AnySignalOpt> {
        match self {
            Self::Float(buffer) => buffer
                .get(index)
                .copied()
                .map(|v| AnySignalOpt::Float(v.into_signal())),
            Self::Int(buffer) => buffer.get(index).copied().map(AnySignalOpt::Int),
            Self::Bool(buffer) => buffer.get(index).copied().map(AnySignalOpt::Bool),
        }
    }

    /// Returns a mutable reference to the signal at the given index.
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<AnySignalOptMut> {
        match self {
            Self::Float(buffer) => buffer.get_mut(index).map(AnySignalOptMut::Float),
            Self::Int(buffer) => buffer.get_mut(index).map(AnySignalOptMut::Int),
            Self::Bool(buffer) => buffer.get_mut(index).map(AnySignalOptMut::Bool),
        }
    }

    /// Returns the signal at the given index.
    #[inline]
    pub fn get_as<S: Signal>(&self, index: usize) -> Option<S> {
        S::try_convert_buffer(self)?
            .get(index)?
            .map(|v| S::from_repr(v))
    }

    /// Returns a mutable reference to the signal at the given index.
    #[inline]
    pub fn get_mut_as<S: Signal>(&mut self, index: usize) -> Option<&mut Option<S::Repr>> {
        S::try_convert_buffer_mut(self)?.get_mut(index)
    }

    /// Sets the signal at the given index.
    ///
    /// # Panics
    ///
    /// Panics if the signal type does not match the buffer type.
    #[inline]
    pub fn set(&mut self, index: usize, value: AnySignalOpt) {
        match (self, value) {
            (Self::Float(buffer), AnySignalOpt::Float(value)) => buffer[index] = value.into_repr(),
            (Self::Int(buffer), AnySignalOpt::Int(value)) => buffer[index] = value.into_repr(),
            (Self::Bool(buffer), AnySignalOpt::Bool(value)) => buffer[index] = value.into_repr(),
            (this, value) => {
                panic!(
                    "Cannot set signal of different type: {:?} != {:?}",
                    this.signal_type(),
                    value.signal_type()
                );
            }
        }
    }

    /// Clones the given signal and stores it at the given index.
    /// Returns `true` if the signal was set successfully.
    #[cfg_attr(feature = "profiling", inline(never))]
    #[cfg_attr(not(feature = "profiling"), inline)]
    pub fn set_as<S: Signal + Clone>(&mut self, index: usize, value: Option<S>) -> bool {
        if let Some(buf) = S::try_convert_buffer_mut(self) {
            let slot = buf.get_mut(index).unwrap();
            slot.clone_from(&value.into_repr()); // `clone_from` is used to possibly avoid cloning the value twice
            true
        } else {
            false
        }
    }

    /// Sets the signal at the given index to `None`.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    #[inline]
    pub fn set_none(&mut self, index: usize) {
        match self {
            Self::Float(buffer) => buffer[index] = None,
            Self::Int(buffer) => buffer[index] = None,
            Self::Bool(buffer) => buffer[index] = None,
        }
    }

    /// Clones the contents of the other buffer into this buffer.
    ///
    /// # Panics
    ///
    /// Panics if the buffer types do not match.
    #[inline]
    pub fn clone_from(&mut self, other: &Self) {
        match (self, other) {
            (Self::Float(this), Self::Float(other)) => {
                this.copy_from_slice(other);
            }
            (Self::Int(this), Self::Int(other)) => {
                this.copy_from_slice(other);
            }
            (Self::Bool(this), Self::Bool(other)) => {
                this.copy_from_slice(other);
            }
            _ => panic!("Cannot copy buffer of different type"),
        }
    }

    /// Returns an iterator over the signals in the buffer.
    #[inline]
    pub fn iter(&self) -> SignalBufferIter {
        SignalBufferIter {
            buffer: self,
            index: 0,
        }
    }

    /// Returns a mutable iterator over the signals in the buffer.
    #[inline]
    pub fn iter_mut(&mut self) -> SignalBufferIterMut {
        SignalBufferIterMut {
            buffer: self,
            index: 0,
            _marker: std::marker::PhantomData,
        }
    }
}

/// An iterator over the signals in a buffer.
pub struct SignalBufferIter<'a> {
    buffer: &'a SignalBuffer,
    index: usize,
}

impl Iterator for SignalBufferIter<'_> {
    type Item = AnySignalOpt;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.buffer.len() {
            let signal = match self.buffer {
                SignalBuffer::Float(buffer) => {
                    AnySignalOpt::Float(buffer[self.index].into_signal())
                }
                SignalBuffer::Int(buffer) => AnySignalOpt::Int(buffer[self.index].into_signal()),
                SignalBuffer::Bool(buffer) => AnySignalOpt::Bool(buffer[self.index].into_signal()),
            };
            self.index += 1;
            Some(signal)
        } else {
            None
        }
    }
}

impl<'a> IntoIterator for &'a SignalBuffer {
    type Item = AnySignalOpt;
    type IntoIter = SignalBufferIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        SignalBufferIter {
            buffer: self,
            index: 0,
        }
    }
}

/// An mutable iterator over the signals in a buffer.
pub struct SignalBufferIterMut<'a> {
    buffer: &'a mut SignalBuffer,
    index: usize,
    _marker: std::marker::PhantomData<AnySignalOptMut<'a>>,
}

impl<'a> Iterator for SignalBufferIterMut<'a> {
    type Item = AnySignalOptMut<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.buffer.len() {
            // SAFETY:
            // We are borrowing the buffer mutably, so we can safely create a mutable reference to the signal.
            // We are also only creating one mutable reference at a time, so there are no issues with aliasing.
            // The lifetime of the mutable reference is limited to the lifetime of the iterator.
            // This is similar to how `std::slice::IterMut` works.
            unsafe {
                let signal = match self.buffer {
                    SignalBuffer::Float(buffer) => AnySignalOptMut::Float(
                        &mut *(&mut buffer[self.index] as *mut OptionRepr<f32>),
                    ),
                    SignalBuffer::Int(buffer) => AnySignalOptMut::Int(
                        &mut *(&mut buffer[self.index] as *mut OptionRepr<i64>),
                    ),
                    SignalBuffer::Bool(buffer) => AnySignalOptMut::Bool(
                        &mut *(&mut buffer[self.index] as *mut OptionRepr<bool>),
                    ),
                };
                self.index += 1;
                Some(signal)
            }
        } else {
            None
        }
    }
}

impl<'a> IntoIterator for &'a mut SignalBuffer {
    type Item = AnySignalOptMut<'a>;
    type IntoIter = SignalBufferIterMut<'a>;

    fn into_iter(self) -> Self::IntoIter {
        SignalBufferIterMut {
            buffer: self,
            index: 0,
            _marker: std::marker::PhantomData,
        }
    }
}

impl FromIterator<f32> for SignalBuffer {
    fn from_iter<T: IntoIterator<Item = f32>>(iter: T) -> Self {
        Self::Float(iter.into_iter().collect())
    }
}

impl FromIterator<i64> for SignalBuffer {
    fn from_iter<T: IntoIterator<Item = i64>>(iter: T) -> Self {
        Self::Int(iter.into_iter().collect())
    }
}

impl FromIterator<bool> for SignalBuffer {
    fn from_iter<T: IntoIterator<Item = bool>>(iter: T) -> Self {
        Self::Bool(iter.into_iter().collect())
    }
}
