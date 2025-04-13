use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
    path::Path,
};

use super::Signal;

/// A contiguous buffer of signals.
#[derive(PartialEq, Clone)]
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
