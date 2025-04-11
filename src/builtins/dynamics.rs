//! Dynamics processors, such as compressors and limiters.

use crate::prelude::*;

/// A simple peak limiter.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `f32` | The input signal. |
/// | `1` | `threshold` | `f32` | The amplitude threshold of the limiter. |
/// | `2` | `attack` | `f32` | The attack factor of the limiter. |
/// | `3` | `release` | `f32` | The release factor of the limiter. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `f32` | The output signal. |
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PeakLimiter {
    gain: f32,
    envelope: f32,

    /// The amplitude threshold of the limiter.
    pub threshold: f32,

    /// The attack factor of the limiter.
    pub attack: f32,

    /// The release factor of the limiter.
    pub release: f32,
}

impl PeakLimiter {
    /// Creates a new `PeakLimiter` processor with the given threshold, attack, and release.
    pub fn new(threshold: f32, attack: f32, release: f32) -> Self {
        Self {
            threshold,
            attack,
            release,
            ..Default::default()
        }
    }
}

impl Default for PeakLimiter {
    fn default() -> Self {
        Self {
            gain: 1.0,
            envelope: 0.0,
            // -0.1 dBFS
            threshold: 0.9885530946569389,
            attack: 0.9,
            release: 0.9995,
        }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for PeakLimiter {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("in", SignalType::Float),
            SignalSpec::new("threshold", SignalType::Float),
            SignalSpec::new("attack", SignalType::Float),
            SignalSpec::new("release", SignalType::Float),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalType::Float)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (in_signal, threshold, attack, release, out) in raug_macros::iter_proc_io_as!(
            inputs as [f32, f32, f32, f32],
            outputs as [f32]
        ) {
            self.threshold = threshold.unwrap_or(self.threshold);
            self.attack = attack.unwrap_or(self.attack);
            self.release = release.unwrap_or(self.release);

            let Some(in_signal) = in_signal else {
                *out = None;
                continue;
            };

            self.envelope = in_signal.abs().max(self.envelope * self.release);

            let target_gain = if self.envelope > self.threshold {
                self.threshold / self.envelope
            } else {
                1.0
            };

            self.gain = self.gain * self.attack + target_gain * (1.0 - self.attack);

            out.set(in_signal * self.gain);
        }

        Ok(())
    }
}

/// A simple compressor.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `f32` | The input signal. |
/// | `1` | `threshold` | `f32` | The amplitude threshold of the compressor. |
/// | `2` | `ratio` | `f32` | The compression ratio of the compressor. |
/// | `3` | `attack` | `f32` | The attack factor of the compressor. |
/// | `4` | `release` | `f32` | The release factor of the compressor. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `f32` | The output signal. |
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Compressor {
    gain: f32,
    envelope: f32,

    /// The amplitude threshold of the compressor.
    pub threshold: f32,

    /// The compression ratio of the compressor.
    pub ratio: f32,

    /// The attack factor of the compressor.
    pub attack: f32,

    /// The release factor of the compressor.
    pub release: f32,
}

impl Compressor {
    /// Creates a new `Compressor` processor with the given threshold, ratio, attack, and release.
    pub fn new(threshold: f32, ratio: f32, attack: f32, release: f32) -> Self {
        Self {
            threshold,
            ratio,
            attack,
            release,
            ..Default::default()
        }
    }
}

impl Default for Compressor {
    fn default() -> Self {
        Self {
            gain: 1.0,
            envelope: 0.0,
            // -0.1 dBFS
            threshold: 0.9885530946569389,
            // 4:1
            ratio: 4.0,
            attack: 0.9,
            release: 0.9995,
        }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for Compressor {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("in", SignalType::Float),
            SignalSpec::new("threshold", SignalType::Float),
            SignalSpec::new("ratio", SignalType::Float),
            SignalSpec::new("attack", SignalType::Float),
            SignalSpec::new("release", SignalType::Float),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalType::Float)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (in_signal, threshold, ratio, attack, release, out) in raug_macros::iter_proc_io_as!(
            inputs as [f32, f32, f32, f32, f32],
            outputs as [f32]
        ) {
            self.threshold = threshold.unwrap_or(self.threshold);
            self.ratio = ratio.unwrap_or(self.ratio);
            self.attack = attack.unwrap_or(self.attack);
            self.release = release.unwrap_or(self.release);

            let Some(in_signal) = in_signal else {
                *out = None;
                continue;
            };

            self.envelope = in_signal.abs().max(self.envelope * self.release);

            let target_gain = if self.envelope > self.threshold {
                self.threshold + (self.envelope - self.threshold) / self.ratio
            } else {
                self.envelope
            };

            self.gain = self.gain * self.attack + target_gain * (1.0 - self.attack);

            out.set(in_signal * self.gain);
        }

        Ok(())
    }
}

/// An RMS compressor.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `f32` | The input signal. |
/// | `1` | `threshold` | `f32` | The amplitude threshold of the compressor. |
/// | `2` | `ratio` | `f32` | The compression ratio of the compressor. |
/// | `3` | `attack` | `f32` | The attack factor of the compressor. |
/// | `4` | `release` | `f32` | The release factor of the compressor. |
/// | `5` | `window_size` | `f32` | The window size of the RMS detector in seconds. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `f32` | The output signal. |
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RmsCompressor {
    gain: f32,
    envelope: f32,
    rms: f32,
    window: Vec<f32>,

    /// The amplitude threshold of the compressor.
    pub threshold: f32,

    /// The compression ratio of the compressor.
    pub ratio: f32,

    /// The attack factor of the compressor.
    pub attack: f32,

    /// The release factor of the compressor.
    pub release: f32,

    /// The window size of the RMS detector in seconds.
    pub window_size: f32,
}

impl RmsCompressor {
    /// Creates a new `RmsCompressor` processor with the given threshold, ratio, attack, release, and window size.
    pub fn new(
        threshold: f32,
        ratio: f32,
        attack: f32,
        release: f32,
        window_size: f32,
    ) -> Self {
        Self {
            threshold,
            ratio,
            attack,
            release,
            window_size,
            ..Default::default()
        }
    }
}

impl Default for RmsCompressor {
    fn default() -> Self {
        Self {
            gain: 1.0,
            envelope: 0.0,
            rms: 0.0,
            window: vec![0.0; 44100],
            // -0.1 dBFS
            threshold: 0.9885530946569389,
            // 4:1
            ratio: 4.0,
            attack: 0.9,
            release: 0.9995,
            window_size: 0.01,
        }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for RmsCompressor {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("in", SignalType::Float),
            SignalSpec::new("threshold", SignalType::Float),
            SignalSpec::new("ratio", SignalType::Float),
            SignalSpec::new("attack", SignalType::Float),
            SignalSpec::new("release", SignalType::Float),
            SignalSpec::new("window_size", SignalType::Int),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalType::Float)]
    }

    fn allocate(&mut self, sample_rate: f32, _max_block_size: usize) {
        self.window
            .resize((self.window_size * sample_rate) as usize, 0.0);
    }

    fn resize_buffers(&mut self, sample_rate: f32, _block_size: usize) {
        self.window.resize(sample_rate as usize, 0.0);
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (in_signal, threshold, ratio, attack, release, window_size, out) in raug_macros::iter_proc_io_as!(
            inputs as [f32, f32, f32, f32, f32, f32],
            outputs as [f32]
        ) {
            self.threshold = threshold.unwrap_or(self.threshold);
            self.ratio = ratio.unwrap_or(self.ratio);
            self.attack = attack.unwrap_or(self.attack);
            self.release = release.unwrap_or(self.release);
            self.window_size = window_size.unwrap_or(self.window_size);

            let Some(in_signal) = in_signal else {
                *out = None;
                continue;
            };

            self.window.rotate_left(1);
            self.window[0] = in_signal.powi(2);

            let window_size = (self.window_size * inputs.sample_rate()) as usize;

            self.rms =
                self.window[..window_size].iter().sum::<f32>() / self.window.len() as f32;
            self.rms = self.rms.sqrt();
            self.envelope = self.rms.max(self.envelope * self.release);

            let target_gain = if self.envelope > self.threshold {
                self.threshold + (self.envelope - self.threshold) / self.ratio
            } else {
                self.envelope
            };

            self.gain = self.gain * self.attack + target_gain * (1.0 - self.attack);

            out.set(in_signal * self.gain);
        }

        Ok(())
    }
}
