//! Dynamics processors, such as compressors and limiters.

use crate::prelude::*;

/// A simple peak limiter.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `Float` | The input signal. |
/// | `1` | `threshold` | `Float` | The amplitude threshold of the limiter. |
/// | `2` | `attack` | `Float` | The attack factor of the limiter. |
/// | `3` | `release` | `Float` | The release factor of the limiter. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Float` | The output signal. |
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PeakLimiter {
    gain: Float,
    envelope: Float,

    /// The amplitude threshold of the limiter.
    pub threshold: Float,

    /// The attack factor of the limiter.
    pub attack: Float,

    /// The release factor of the limiter.
    pub release: Float,
}

impl PeakLimiter {
    /// Creates a new `PeakLimiter` processor with the given threshold, attack, and release.
    pub fn new(threshold: Float, attack: Float, release: Float) -> Self {
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
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        let (in_signal, threshold, attack, release) =
            inputs.as_tuple::<(Float, Float, Float, Float)>()?;

        self.threshold = threshold.unwrap_or(self.threshold);
        self.attack = attack.unwrap_or(self.attack);
        self.release = release.unwrap_or(self.release);

        let Some(in_signal) = in_signal else {
            outputs.set_output_none(0);
            return Ok(());
        };

        self.envelope = in_signal.abs().max(self.envelope * self.release);

        let target_gain = if self.envelope > self.threshold {
            self.threshold / self.envelope
        } else {
            1.0
        };

        self.gain = self.gain * self.attack + target_gain * (1.0 - self.attack);

        outputs.set_output_as(0, in_signal * self.gain)?;

        Ok(())
    }
}

/// A simple compressor.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `Float` | The input signal. |
/// | `1` | `threshold` | `Float` | The amplitude threshold of the compressor. |
/// | `2` | `ratio` | `Float` | The compression ratio of the compressor. |
/// | `3` | `attack` | `Float` | The attack factor of the compressor. |
/// | `4` | `release` | `Float` | The release factor of the compressor. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Float` | The output signal. |
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Compressor {
    gain: Float,
    envelope: Float,

    /// The amplitude threshold of the compressor.
    pub threshold: Float,

    /// The compression ratio of the compressor.
    pub ratio: Float,

    /// The attack factor of the compressor.
    pub attack: Float,

    /// The release factor of the compressor.
    pub release: Float,
}

impl Compressor {
    /// Creates a new `Compressor` processor with the given threshold, ratio, attack, and release.
    pub fn new(threshold: Float, ratio: Float, attack: Float, release: Float) -> Self {
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
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        let (in_signal, threshold, ratio, attack, release) =
            inputs.as_tuple::<(Float, Float, Float, Float, Float)>()?;

        self.threshold = threshold.unwrap_or(self.threshold);
        self.ratio = ratio.unwrap_or(self.ratio);
        self.attack = attack.unwrap_or(self.attack);
        self.release = release.unwrap_or(self.release);

        let Some(in_signal) = in_signal else {
            outputs.set_output_none(0);
            return Ok(());
        };

        self.envelope = in_signal.abs().max(self.envelope * self.release);

        let target_gain = if self.envelope > self.threshold {
            self.threshold + (self.envelope - self.threshold) / self.ratio
        } else {
            self.envelope
        };

        self.gain = self.gain * self.attack + target_gain * (1.0 - self.attack);

        outputs.set_output_as(0, in_signal * self.gain)?;

        Ok(())
    }
}

/// An RMS compressor.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `Float` | The input signal. |
/// | `1` | `threshold` | `Float` | The amplitude threshold of the compressor. |
/// | `2` | `ratio` | `Float` | The compression ratio of the compressor. |
/// | `3` | `attack` | `Float` | The attack factor of the compressor. |
/// | `4` | `release` | `Float` | The release factor of the compressor. |
/// | `5` | `window_size` | `Float` | The window size of the RMS detector in seconds. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Float` | The output signal. |
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RmsCompressor {
    gain: Float,
    envelope: Float,
    rms: Float,
    window: Vec<Float>,

    /// The amplitude threshold of the compressor.
    pub threshold: Float,

    /// The compression ratio of the compressor.
    pub ratio: Float,

    /// The attack factor of the compressor.
    pub attack: Float,

    /// The release factor of the compressor.
    pub release: Float,

    /// The window size of the RMS detector in seconds.
    pub window_size: Float,
}

impl RmsCompressor {
    /// Creates a new `RmsCompressor` processor with the given threshold, ratio, attack, release, and window size.
    pub fn new(
        threshold: Float,
        ratio: Float,
        attack: Float,
        release: Float,
        window_size: Float,
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

    fn allocate(&mut self, sample_rate: Float, _max_block_size: usize) {
        self.window
            .resize((self.window_size * sample_rate) as usize, 0.0);
    }

    fn resize_buffers(&mut self, sample_rate: Float, _block_size: usize) {
        self.window.resize(sample_rate as usize, 0.0);
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        let (in_signal, threshold, ratio, attack, release, window_size) =
            inputs.as_tuple::<(Float, Float, Float, Float, Float, Float)>()?;

        self.threshold = threshold.unwrap_or(self.threshold);
        self.ratio = ratio.unwrap_or(self.ratio);
        self.attack = attack.unwrap_or(self.attack);
        self.release = release.unwrap_or(self.release);
        self.window_size = window_size.unwrap_or(self.window_size);

        let Some(in_signal) = in_signal else {
            outputs.set_output_none(0);
            return Ok(());
        };

        self.window.rotate_left(1);
        self.window[0] = in_signal.powi(2);

        let window_size = (self.window_size * inputs.sample_rate()) as usize;

        self.rms = self.window[..window_size].iter().sum::<Float>() / self.window.len() as Float;
        self.rms = self.rms.sqrt();
        self.envelope = self.rms.max(self.envelope * self.release);

        let target_gain = if self.envelope > self.threshold {
            self.threshold + (self.envelope - self.threshold) / self.ratio
        } else {
            self.envelope
        };

        self.gain = self.gain * self.attack + target_gain * (1.0 - self.attack);

        outputs.set_output_as(0, in_signal * self.gain)?;

        Ok(())
    }
}
