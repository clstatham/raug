//! Fast Fourier Transform (FFT) processing.

use signal::{FftSignalType, RealBuf};

use crate::prelude::*;

pub mod builder;
pub mod builtins;
pub mod graph;
pub mod processor;
pub mod signal;

/// An error that can occur during FFT processing.
#[derive(Debug, Clone, thiserror::Error)]
pub enum FftError {
    /// An error occurred during an FFT operation.
    #[error("realfft error: {0}")]
    RealFft(String),
    #[error("invalid signal type: {0:?}, expected {1:?}")]
    InvalidSignalType(FftSignalType, FftSignalType),
}

impl From<realfft::FftError> for FftError {
    fn from(err: realfft::FftError) -> Self {
        Self::RealFft(err.to_string())
    }
}

/// A window function to apply to the input signal before FFT processing.
#[derive(Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum WindowFunction {
    /// A rectangular window function (no windowing).
    Rectangular,
    /// A Hann window function.
    #[default]
    Hann,
    /// A Hamming window function.
    Hamming,
    /// A Blackman window function.
    Blackman,
    /// A Nuttall window function.
    Nuttall,
    /// A triangular window function.
    Triangular,
}

impl WindowFunction {
    /// Generates a window of the given length using this window function.
    pub fn generate(&self, length: usize) -> RealBuf {
        let mut buf = vec![0.0 as f32; length].into_boxed_slice();
        match self {
            Self::Rectangular => {
                for x in buf.iter_mut() {
                    *x = 1.0;
                }
            }
            Self::Hann => {
                buf = apodize::hanning_iter(length).map(|x| x as f32).collect();
            }
            Self::Hamming => {
                buf = apodize::hamming_iter(length).map(|x| x as f32).collect();
            }
            Self::Blackman => {
                buf = apodize::blackman_iter(length).map(|x| x as f32).collect();
            }
            Self::Nuttall => {
                buf = apodize::nuttall_iter(length).map(|x| x as f32).collect();
            }
            Self::Triangular => {
                buf = apodize::triangular_iter(length)
                    .map(|x| x as f32)
                    .collect();
            }
        }
        RealBuf(buf)
    }
}
