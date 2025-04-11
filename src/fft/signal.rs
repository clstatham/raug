//! FFT signal types.

use num::Complex;

use crate::prelude::*;
use std::ops::{AddAssign, Deref, DerefMut, MulAssign};

/// A buffer of real numbers.
///
/// This differs from [`Buffer<f32>`](crate::signal::Buffer) in that it is does not internally store [`Option`]s - every element is guaranteed to have value.
/// It also cannot be resized, pushed to, or popped from.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RealBuf(pub(crate) Box<[f32]>);

impl RealBuf {
    /// Creates a new `RealBuf` with the given length.
    pub fn new(length: usize) -> Self {
        Self(vec![0.0; length].into_boxed_slice())
    }
}

impl Deref for RealBuf {
    type Target = [f32];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RealBuf {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AsRef<[f32]> for RealBuf {
    fn as_ref(&self) -> &[f32] {
        &self.0
    }
}

impl AsMut<[f32]> for RealBuf {
    fn as_mut(&mut self) -> &mut [f32] {
        &mut self.0
    }
}

impl AddAssign<f32> for RealBuf {
    fn add_assign(&mut self, rhs: f32) {
        for x in self.iter_mut() {
            *x += rhs;
        }
    }
}

impl AddAssign<&Self> for RealBuf {
    fn add_assign(&mut self, rhs: &Self) {
        for (x, y) in self.iter_mut().zip(rhs.iter()) {
            *x += *y;
        }
    }
}

impl MulAssign<f32> for RealBuf {
    fn mul_assign(&mut self, rhs: f32) {
        for x in self.iter_mut() {
            *x *= rhs;
        }
    }
}

impl MulAssign<&Self> for RealBuf {
    fn mul_assign(&mut self, rhs: &Self) {
        for (x, y) in self.iter_mut().zip(rhs.iter()) {
            *x *= *y;
        }
    }
}

impl FromIterator<f32> for RealBuf {
    fn from_iter<I: IntoIterator<Item = f32>>(iter: I) -> Self {
        Self(iter.into_iter().collect::<Vec<_>>().into_boxed_slice())
    }
}

impl<'a> IntoIterator for &'a RealBuf {
    type Item = &'a f32;
    type IntoIter = std::slice::Iter<'a, f32>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a mut RealBuf {
    type Item = &'a mut f32;
    type IntoIter = std::slice::IterMut<'a, f32>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

/// A buffer of complex numbers.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ComplexBuf(pub(crate) Box<[Complex<f32>]>);

impl ComplexBuf {
    /// Creates a new `Fft` for the given FFT length.
    ///
    /// Since this is a real-to-complex FFT, the length of the output is `fft_length / 2 + 1`.
    pub fn new_for_real_length(fft_length: usize) -> Self {
        let complex_length = fft_length / 2 + 1;
        Self(vec![Complex::default(); complex_length].into_boxed_slice())
    }
}

impl Deref for ComplexBuf {
    type Target = [Complex<f32>];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ComplexBuf {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AsRef<[Complex<f32>]> for ComplexBuf {
    fn as_ref(&self) -> &[Complex<f32>] {
        &self.0
    }
}

impl AsMut<[Complex<f32>]> for ComplexBuf {
    fn as_mut(&mut self) -> &mut [Complex<f32>] {
        &mut self.0
    }
}

impl FromIterator<Complex<f32>> for ComplexBuf {
    fn from_iter<I: IntoIterator<Item = Complex<f32>>>(iter: I) -> Self {
        Self(iter.into_iter().collect::<Vec<_>>().into_boxed_slice())
    }
}

impl<'a> IntoIterator for &'a ComplexBuf {
    type Item = &'a Complex<f32>;
    type IntoIter = std::slice::Iter<'a, Complex<f32>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a mut ComplexBuf {
    type Item = &'a mut Complex<f32>;
    type IntoIter = std::slice::IterMut<'a, Complex<f32>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FftBufLength {
    FftLength,
    PaddedLength,
    FftLengthPlusOne,
    BlockSize,
    Custom(usize),
}

impl FftBufLength {
    pub fn calculate(&self, fft_length: usize, block_size: usize) -> usize {
        match self {
            FftBufLength::FftLength => fft_length,
            FftBufLength::PaddedLength => fft_length * 2,
            FftBufLength::FftLengthPlusOne => fft_length + 1,
            FftBufLength::BlockSize => block_size,
            FftBufLength::Custom(length) => *length,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FftSignalType {
    Param(SignalType),
    RealBuf(FftBufLength),
    ComplexBuf(FftBufLength),
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FftSignal {
    Param(Param),
    RealBuf(RealBuf),
    ComplexBuf(ComplexBuf),
}

impl FftSignal {
    pub fn signal_type(&self) -> FftSignalType {
        match self {
            FftSignal::Param(param) => FftSignalType::Param(param.signal_type()),
            FftSignal::RealBuf(buf) => FftSignalType::RealBuf(FftBufLength::Custom(buf.len())),
            FftSignal::ComplexBuf(buf) => {
                FftSignalType::ComplexBuf(FftBufLength::Custom(buf.len()))
            }
        }
    }

    pub fn as_param(&self) -> Option<&Param> {
        match self {
            FftSignal::Param(param) => Some(param),
            _ => None,
        }
    }

    pub fn as_real_buf(&self) -> Option<&RealBuf> {
        match self {
            FftSignal::RealBuf(buf) => Some(buf),
            _ => None,
        }
    }

    pub fn as_complex_buf(&self) -> Option<&ComplexBuf> {
        match self {
            FftSignal::ComplexBuf(buf) => Some(buf),
            _ => None,
        }
    }

    pub fn as_real_buf_mut(&mut self) -> Option<&mut RealBuf> {
        match self {
            FftSignal::RealBuf(buf) => Some(buf),
            _ => None,
        }
    }

    pub fn as_complex_buf_mut(&mut self) -> Option<&mut ComplexBuf> {
        match self {
            FftSignal::ComplexBuf(buf) => Some(buf),
            _ => None,
        }
    }
}
