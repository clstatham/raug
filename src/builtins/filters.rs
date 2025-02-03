//! Built-in filters for processing audio signals.

use crate::{prelude::*, signal::PI};

const THERMAL: Float = 0.000025;

/// A 4-pole Moog ladder lowpass filter.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `Float` | The input signal. |
/// | `1` | `cutoff` | `Float` | The cutoff frequency of the filter. |
/// | `2` | `resonance` | `Float` | The resonance of the filter. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Float` | The output signal. |
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MoogLadder {
    stage: [Float; 4],
    stage_tanh: [Float; 3],
    delay: [Float; 6],
    tune: Float,
    acr: Float,
    res_quad: Float,

    /// The cutoff frequency of the filter.
    pub cutoff: Float,

    /// The resonance of the filter.
    pub resonance: Float,
}

impl Default for MoogLadder {
    fn default() -> Self {
        Self {
            stage: [0.0; 4],
            stage_tanh: [0.0; 3],
            delay: [0.0; 6],
            tune: 0.0,
            acr: 0.0,
            res_quad: 0.0,
            cutoff: 1000.0,
            resonance: 0.1,
        }
    }
}

impl MoogLadder {
    /// Creates a new `MoogLadder` filter with the given cutoff frequency and resonance.
    pub fn new(cutoff: Float, resonance: Float) -> Self {
        Self {
            cutoff,
            resonance,
            ..Default::default()
        }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for MoogLadder {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("in", SignalType::Float),
            SignalSpec::new("cutoff", SignalType::Float),
            SignalSpec::new("resonance", SignalType::Float),
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
        // based on: https://github.com/ddiakopoulos/MoogLadders/blob/fd147415573e723ba102dfc63dc46af0b7fe55b9/src/HuovilainenModel.h
        for (in_signal, cutoff, resonance, out) in iter_proc_io!(
            inputs as [Float, Float, Float],
            outputs as [Float]
        ) {
            let Some(in_signal) = in_signal else {
                *out = None;
                continue;
            };

            if let Some(cutoff) = cutoff {
                self.cutoff = cutoff.clamp(0.0, inputs.sample_rate() * 0.5);
            }

            if let Some(resonance) = resonance {
                self.resonance = resonance.clamp(0.0, 1.0);
            }

            let fc = self.cutoff / inputs.sample_rate();
            let f = fc * 0.5; // oversampling
            let fc2 = fc * fc;
            let fc3 = fc2 * fc;

            let fcr = 1.8730 * fc3 + 0.4955 * fc2 - 0.6490 * fc + 0.9988;
            self.acr = -3.9364 * fc2 + 1.8409 * fc + 0.9968;
            self.tune = (1.0 - Float::exp(-((2.0 * PI) * f * fcr))) / THERMAL;
            self.res_quad = 4.0 * self.resonance * self.acr;

            // oversample
            for _ in 0..2 {
                let mut inp = in_signal - self.res_quad * self.delay[5];
                self.stage[0] =
                    self.delay[0] + self.tune * (Float::tanh(inp * THERMAL) - self.stage_tanh[0]);
                self.delay[0] = self.stage[0];
                for k in 1..4 {
                    inp = self.stage[k - 1];
                    self.stage_tanh[k - 1] = Float::tanh(inp * THERMAL);
                    if k == 3 {
                        self.stage[k] = self.delay[k]
                            + self.tune
                                * (self.stage_tanh[k - 1] - Float::tanh(self.delay[k] * THERMAL));
                    } else {
                        self.stage[k] = self.delay[k]
                            + self.tune * (self.stage_tanh[k - 1] - self.stage_tanh[k]);
                    }
                    self.delay[k] = self.stage[k];
                }
                self.delay[5] = (self.stage[3] + self.delay[4]) * 0.5;
                self.delay[4] = self.stage[3];
            }

            *out = Some(self.delay[5]);
        }

        Ok(())
    }
}

/// A biquad filter with configurable coefficients.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `Float` | The input signal. |
/// | `1` | `a0` | `Float` | The `a0` coefficient. |
/// | `2` | `a1` | `Float` | The `a1` coefficient. |
/// | `3` | `a2` | `Float` | The `a2` coefficient. |
/// | `4` | `b1` | `Float` | The `b1` coefficient. |
/// | `5` | `b2` | `Float` | The `b2` coefficient. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Float` | The output signal. |
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Biquad {
    /// The `a0` coefficient.
    pub a0: Float,

    /// The `a1` coefficient.
    pub a1: Float,

    /// The `a2` coefficient.
    pub a2: Float,

    /// The `b1` coefficient.
    pub b1: Float,

    /// The `b2` coefficient.
    pub b2: Float,

    // input state
    x1: Float,
    x2: Float,

    // output state
    y1: Float,
    y2: Float,
}

impl Default for Biquad {
    fn default() -> Self {
        Self {
            a0: 1.0,
            a1: 0.0,
            a2: 0.0,
            b1: 0.0,
            b2: 0.0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }
}

impl Biquad {
    /// Creates a new `Biquad` filter with the given coefficients.
    pub fn new(a0: Float, a1: Float, a2: Float, b1: Float, b2: Float) -> Self {
        Self {
            a0,
            a1,
            a2,
            b1,
            b2,
            ..Default::default()
        }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for Biquad {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("in", SignalType::Float),
            SignalSpec::new("a0", SignalType::Float),
            SignalSpec::new("a1", SignalType::Float),
            SignalSpec::new("a2", SignalType::Float),
            SignalSpec::new("b1", SignalType::Float),
            SignalSpec::new("b2", SignalType::Float),
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
        for (in_signal, a0, a1, a2, b1, b2, out) in iter_proc_io!(
            inputs as [Float, Float, Float, Float, Float, Float],
            outputs as [Float]
        ) {
            let Some(in_signal) = in_signal else {
                *out = None;
                continue;
            };

            if let Some(a0) = a0 {
                self.a0 = *a0;
            }
            if let Some(a1) = a1 {
                self.a1 = *a1;
            }
            if let Some(a2) = a2 {
                self.a2 = *a2;
            }
            if let Some(b1) = b1 {
                self.b1 = *b1;
            }
            if let Some(b2) = b2 {
                self.b2 = *b2;
            }

            let filtered = self.a0 * in_signal + self.a1 * self.x1 + self.a2 * self.x2
                - self.b1 * self.y1
                - self.b2 * self.y2;

            self.x2 = self.x1;
            self.x1 = *in_signal;
            self.y2 = self.y1;
            self.y1 = filtered;

            *out = Some(filtered);
        }

        Ok(())
    }
}

/// A type of biquad filter.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BiquadType {
    /// A lowpass filter.
    LowPass,
    /// A highpass filter.
    HighPass,
    /// A bandpass filter.
    BandPass,
    /// A notch filter.
    Notch,
    /// An equalizer peak filter.
    Peak,
    /// An equalizer low shelf filter.
    LowShelf,
    /// An equalizer high shelf filter.
    HighShelf,
}

impl TryFrom<&str> for BiquadType {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "lowpass" => Ok(Self::LowPass),
            "highpass" => Ok(Self::HighPass),
            "bandpass" => Ok(Self::BandPass),
            "notch" => Ok(Self::Notch),
            "peak" => Ok(Self::Peak),
            "lowshelf" => Ok(Self::LowShelf),
            "highshelf" => Ok(Self::HighShelf),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for BiquadType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BiquadType::LowPass => write!(f, "lowpass"),
            BiquadType::HighPass => write!(f, "highpass"),
            BiquadType::BandPass => write!(f, "bandpass"),
            BiquadType::Notch => write!(f, "notch"),
            BiquadType::Peak => write!(f, "peak"),
            BiquadType::LowShelf => write!(f, "lowshelf"),
            BiquadType::HighShelf => write!(f, "highshelf"),
        }
    }
}

/// A bi-quad filter with automatic coefficient calculation.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AutoBiquad {
    // biquad state
    a0: Float,
    a1: Float,
    a2: Float,
    b1: Float,
    b2: Float,
    x1: Float,
    x2: Float,
    y1: Float,
    y2: Float,

    // the type of biquad filter
    biquad_type: BiquadType,

    /// The cutoff frequency of the filter.
    pub cutoff: Float,

    /// The Q factor of the filter.
    pub q: Float,

    /// The gain of the filter.
    pub gain: Float,
}

impl AutoBiquad {
    /// Creates a new `AutoBiquad` filter with the given type, cutoff frequency, Q factor, and gain.
    pub fn new(biquad_type: BiquadType, cutoff: Float, q: Float, gain: Float) -> Self {
        Self {
            biquad_type,
            cutoff,
            q,
            gain,
            a0: 1.0,
            a1: 0.0,
            a2: 0.0,
            b1: 0.0,
            b2: 0.0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    /// Creates a new lowpass `AutoBiquad` filter with the given cutoff frequency and Q factor.
    pub fn lowpass(cutoff: Float, q: Float) -> Self {
        Self::new(BiquadType::LowPass, cutoff, q, 0.0)
    }

    /// Creates a new highpass `AutoBiquad` filter with the given cutoff frequency and Q factor.
    pub fn highpass(cutoff: Float, q: Float) -> Self {
        Self::new(BiquadType::HighPass, cutoff, q, 0.0)
    }

    /// Creates a new bandpass `AutoBiquad` filter with the given cutoff frequency and Q factor.
    pub fn bandpass(cutoff: Float, q: Float) -> Self {
        Self::new(BiquadType::BandPass, cutoff, q, 0.0)
    }

    /// Creates a new notch `AutoBiquad` filter with the given cutoff frequency and Q factor.
    pub fn notch(cutoff: Float, q: Float) -> Self {
        Self::new(BiquadType::Notch, cutoff, q, 0.0)
    }

    /// Creates a new peak `AutoBiquad` filter with the given cutoff frequency, Q factor, and gain.
    pub fn peak(cutoff: Float, q: Float, gain: Float) -> Self {
        Self::new(BiquadType::Peak, cutoff, q, gain)
    }

    /// Creates a new low shelf `AutoBiquad` filter with the given cutoff frequency, Q factor, and gain.
    pub fn low_shelf(cutoff: Float, q: Float, gain: Float) -> Self {
        Self::new(BiquadType::LowShelf, cutoff, q, gain)
    }

    /// Creates a new high shelf `AutoBiquad` filter with the given cutoff frequency, Q factor, and gain.
    pub fn high_shelf(cutoff: Float, q: Float, gain: Float) -> Self {
        Self::new(BiquadType::HighShelf, cutoff, q, gain)
    }

    /// Returns the type of biquad filter this is.
    pub fn biquad_type(&self) -> BiquadType {
        self.biquad_type
    }

    // http://www.earlevel.com/scripts/widgets/20131013/biquads2.js
    #[inline]
    fn set_coefficients(&mut self, sample_rate: Float) {
        if self.q < 0.01 {
            self.q = 0.01;
        }

        let v = Float::powf(10.0, self.gain.abs() / 20.0);
        let k = Float::tan(PI * self.cutoff / sample_rate);

        match self.biquad_type {
            BiquadType::LowPass => {
                let norm = 1.0 / (1.0 + k / self.q + k * k);
                self.a0 = k * k * norm;
                self.a1 = 2.0 * self.a0;
                self.a2 = self.a0;
                self.b1 = 2.0 * (k * k - 1.0) * norm;
                self.b2 = (1.0 - k / self.q + k * k) * norm;
            }
            BiquadType::HighPass => {
                let norm = 1.0 / (1.0 + k / self.q + k * k);
                self.a0 = 1.0 * norm;
                self.a1 = -2.0 * self.a0;
                self.a2 = self.a0;
                self.b1 = 2.0 * (k * k - 1.0) * norm;
                self.b2 = (1.0 - k / self.q + k * k) * norm;
            }
            BiquadType::BandPass => {
                let norm = 1.0 / (1.0 + k / self.q + k * k);
                self.a0 = k / self.q * norm;
                self.a1 = 0.0;
                self.a2 = -self.a0;
                self.b1 = 2.0 * (k * k - 1.0) * norm;
                self.b2 = (1.0 - k / self.q + k * k) * norm;
            }
            BiquadType::Notch => {
                let norm = 1.0 / (1.0 + k / self.q + k * k);
                self.a0 = (1.0 + k * k) * norm;
                self.a1 = 2.0 * (k * k - 1.0) * norm;
                self.a2 = self.a0;
                self.b1 = self.a1;
                self.b2 = (1.0 - k / self.q + k * k) * norm;
            }
            BiquadType::Peak => {
                if self.gain >= 0.0 {
                    let norm = 1.0 / (1.0 + 1.0 / self.q * k + k * k);
                    self.a0 = (1.0 + v / self.q * k + k * k) * norm;
                    self.a1 = 2.0 * (k * k - 1.0) * norm;
                    self.a2 = (1.0 - v / self.q * k + k * k) * norm;
                    self.b1 = self.a1;
                    self.b2 = (1.0 - 1.0 / self.q * k + k * k) * norm;
                } else {
                    let norm = 1.0 / (1.0 + v / self.q * k + k * k);
                    self.a0 = (1.0 + 1.0 / self.q * k + k * k) * norm;
                    self.a1 = 2.0 * (k * k - 1.0) * norm;
                    self.a2 = (1.0 - 1.0 / self.q * k + k * k) * norm;
                    self.b1 = self.a1;
                    self.b2 = (1.0 - v / self.q * k + k * k) * norm;
                }
            }
            BiquadType::LowShelf => {
                if self.gain >= 0.0 {
                    let norm = 1.0 / (1.0 + Float::sqrt(2.0) * k + k * k);
                    self.a0 = (1.0 + Float::sqrt(2.0 * v) * k + v * k * k) * norm;
                    self.a1 = 2.0 * (v * k * k - 1.0) * norm;
                    self.a2 = (1.0 - Float::sqrt(2.0 * v) * k + v * k * k) * norm;
                    self.b1 = 2.0 * (k * k - 1.0) * norm;
                    self.b2 = (1.0 - Float::sqrt(2.0) * k + k * k) * norm;
                } else {
                    let norm = 1.0 / (1.0 + Float::sqrt(2.0) * k + k * k);
                    self.a0 = (v + Float::sqrt(2.0 * v) * k + k * k) * norm;
                    self.a1 = 2.0 * (k * k - v) * norm;
                    self.a2 = (v - Float::sqrt(2.0 * v) * k + k * k) * norm;
                    self.b1 = 2.0 * (k * k - 1.0) * norm;
                    self.b2 = (1.0 - Float::sqrt(2.0) * k + k * k) * norm;
                }
            }
            BiquadType::HighShelf => {
                if self.gain >= 0.0 {
                    let norm = 1.0 / (1.0 + Float::sqrt(2.0) * k + k * k);
                    self.a0 = (v + Float::sqrt(2.0 * v) * k + k * k) * norm;
                    self.a1 = 2.0 * (k * k - v) * norm;
                    self.a2 = (v - Float::sqrt(2.0 * v) * k + k * k) * norm;
                    self.b1 = 2.0 * (k * k - 1.0) * norm;
                    self.b2 = (1.0 - Float::sqrt(2.0) * k + k * k) * norm;
                } else {
                    let norm = 1.0 / (v + Float::sqrt(2.0 * v) * k + k * k);
                    self.a0 = (1.0 + Float::sqrt(2.0) * k + k * k) * norm;
                    self.a1 = 2.0 * (k * k - 1.0) * norm;
                    self.a2 = (1.0 - Float::sqrt(2.0) * k + k * k) * norm;
                    self.b1 = 2.0 * (v * k * k - 1.0) * norm;
                    self.b2 = (v - Float::sqrt(2.0 * v) * k + v * k * k) * norm;
                }
            }
        }

        #[cfg(debug_assertions)]
        if sample_rate > 0.0 {
            // check for NaN
            assert!(self.a0.is_finite(), "biquad: malformed a0 coefficient");
            assert!(self.a1.is_finite(), "biquad: malformed a1 coefficient");
            assert!(self.a2.is_finite(), "biquad: malformed a2 coefficient");
            assert!(self.b1.is_finite(), "biquad: malformed b1 coefficient");
            assert!(self.b2.is_finite(), "biquad: malformed b2 coefficient");
        }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for AutoBiquad {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("in", SignalType::Float),
            SignalSpec::new("frequency", SignalType::Float),
            SignalSpec::new("q", SignalType::Float),
            SignalSpec::new("gain", SignalType::Float),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalType::Float)]
    }

    fn resize_buffers(&mut self, sample_rate: Float, _block_size: usize) {
        self.set_coefficients(sample_rate);
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (in_signal, frequency, q, gain, out) in iter_proc_io!(
            inputs as [Float, Float, Float, Float],
            outputs as [Float]
        ) {
            let Some(in_signal) = in_signal else {
                *out = None;
                continue;
            };

            let frequency = frequency.unwrap_or(self.cutoff);
            let q = q.unwrap_or(self.q);
            let gain = gain.unwrap_or(self.gain);

            let frequency_changed = (frequency - self.cutoff).abs() > Float::EPSILON;
            let q_changed = (q - self.q).abs() > Float::EPSILON;
            let gain_changed = (gain - self.gain).abs() > Float::EPSILON;

            if frequency_changed || q_changed || gain_changed {
                self.cutoff = frequency;
                self.q = q;
                self.gain = gain;

                self.set_coefficients(inputs.sample_rate());
            }

            let filtered = self.a0 * in_signal + self.a1 * self.x1 + self.a2 * self.x2
                - self.b1 * self.y1
                - self.b2 * self.y2;

            self.x2 = self.x1;
            self.x1 = *in_signal;
            self.y2 = self.y1;
            self.y1 = filtered;

            *out = Some(filtered);
        }

        Ok(())
    }
}

/// A 1-pole lowpass filter.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `Float` | The input signal. |
/// | `1` | `cutoff` | `Float` | The cutoff frequency of the filter. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Float` | The output signal. |
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OnePole {
    cutoff: Float,
    a0: Float,
    b1: Float,
    x1: Float,
}

impl Default for OnePole {
    fn default() -> Self {
        Self {
            cutoff: 1000.0,
            a0: 1.0,
            b1: 0.0,
            x1: 0.0,
        }
    }
}

impl OnePole {
    /// Creates a new `OnePole` filter with the given cutoff frequency.
    pub fn new(cutoff: Float) -> Self {
        Self {
            cutoff,
            ..Default::default()
        }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for OnePole {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("in", SignalType::Float),
            SignalSpec::new("cutoff", SignalType::Float),
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
        for (in_signal, cutoff, out) in iter_proc_io!(
            inputs as [Float, Float],
            outputs as [Float]
        ) {
            self.cutoff = cutoff
                .unwrap_or(self.cutoff)
                .clamp(0.0, inputs.sample_rate() * 0.5);
            self.b1 = Float::exp(-2.0 * PI * self.cutoff / inputs.sample_rate());
            self.a0 = 1.0 - self.b1;

            let Some(in_signal) = in_signal else {
                *out = None;
                continue;
            };

            let filtered = self.a0 * in_signal + self.b1 * self.x1;

            self.x1 = *in_signal;

            *out = Some(filtered);
        }

        Ok(())
    }
}
