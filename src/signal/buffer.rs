use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
    path::Path,
};

use super::{Float, Signal, optional::Repr};

/// A contiguous buffer of signals.
///
/// The signals are stored as a [`Vec`] of [`Option<T>`] to allow for missing values.
///
/// This type implements [`Deref`] and [`DerefMut`] so that it can be used as a slice of [`Option<T>`].
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

impl Buffer<Float> {
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
                writer.write_sample(sample.into_signal() as f32)?;
            } else {
                writer.write_sample(0.0)?;
            }
        }
        writer.finalize()?;
        Ok(())
    }

    /// Returns the maximum value in the buffer out of all entries that are [`Some`].
    ///
    /// If the buffer is empty, this returns [`Float::MIN`].
    #[inline]
    pub fn max(&self) -> Float {
        self.buf
            .iter()
            .flatten()
            .fold(Float::MIN, |a, b| a.max(b.into_signal()))
    }

    /// Returns the minimum value in the buffer out of all entries that are [`Some`].
    ///
    /// If the buffer is empty, this returns [`Float::MAX`].
    #[inline]
    pub fn min(&self) -> Float {
        self.buf
            .iter()
            .flatten()
            .fold(Float::MAX, |a, b| a.min(b.into_signal()))
    }

    /// Returns the sum of all entries that are [`Some`].
    ///
    /// If the buffer is empty, this returns `0.0`.
    #[inline]
    pub fn sum(&self) -> Float {
        self.buf
            .iter()
            .flatten()
            .fold(0.0, |a, b| a + b.into_signal())
    }

    /// Returns the mean of all entries that are [`Some`].
    ///
    /// # Panics
    ///
    /// Panics if the buffer is empty.
    #[inline]
    pub fn mean(&self) -> Float {
        self.sum() / self.len() as Float
    }

    /// Returns the root mean square of all entries that are [`Some`].
    ///
    /// If the buffer is empty, this returns `0.0`.
    #[inline]
    pub fn rms(&self) -> Float {
        self.buf
            .iter()
            .flatten()
            .fold(0.0, |a, b| a + b.into_signal() * b.into_signal())
            .sqrt()
    }

    /// Returns the variance of all entries that are [`Some`].
    ///
    /// If the buffer has less than 2 entries, this returns `0.0`.
    #[inline]
    pub fn variance(&self) -> Float {
        if self.len() < 2 {
            return 0.0;
        }
        let mean = self.mean();
        let sum = self.buf.iter().flatten().fold(0.0, |a, b| {
            a + (b.into_signal() - mean) * (b.into_signal() - mean)
        });
        sum / (self.len() - 1) as Float
    }

    /// Returns the standard deviation of all entries that are [`Some`].
    ///
    /// This is the square root of the variance.
    #[inline]
    pub fn stddev(&self) -> Float {
        self.variance().sqrt()
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
