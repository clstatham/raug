//! Built-in filters for processing audio signals.

use std::f32::consts::PI;

use crate::prelude::*;

const THERMAL: f32 = 0.000025;

/// A 4-pole Moog ladder lowpass filter.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `f32` | The input signal. |
/// | `1` | `cutoff` | `f32` | The cutoff frequency of the filter. |
/// | `2` | `resonance` | `f32` | The resonance of the filter. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `f32` | The output signal. |
#[derive(Clone, Debug, Processor)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", processor_typetag)]
pub struct MoogLadder {
    stage: [f32; 4],
    stage_tanh: [f32; 3],
    delay: [f32; 6],
    tune: f32,
    acr: f32,
    res_quad: f32,

    #[input]
    input: f32,
    #[input]
    cutoff: f32,
    #[input]
    resonance: f32,

    #[output]
    output: f32,
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
            input: 0.0,
            cutoff: 1000.0,
            resonance: 0.1,
            output: 0.0,
        }
    }
}

impl MoogLadder {
    /// Creates a new `MoogLadder` filter with the given cutoff frequency and resonance.
    pub fn new(cutoff: f32, resonance: f32) -> Self {
        Self {
            cutoff,
            resonance,
            ..Default::default()
        }
    }

    pub fn update(&mut self, env: &ProcEnv) {
        self.cutoff = self.cutoff.clamp(0.0, env.sample_rate * 0.5);
        self.resonance = self.resonance.clamp(0.0, 1.0);

        let fc = self.cutoff / env.sample_rate;
        let f = fc * 0.5; // oversampling
        let fc2 = fc * fc;
        let fc3 = fc2 * fc;

        let fcr = 1.8730 * fc3 + 0.4955 * fc2 - 0.6490 * fc + 0.9988;
        self.acr = -3.9364 * fc2 + 1.8409 * fc + 0.9968;
        self.tune = (1.0 - f32::exp(-((2.0 * PI) * f * fcr))) / THERMAL;
        self.res_quad = 4.0 * self.resonance * self.acr;

        // oversample
        for _ in 0..2 {
            let mut inp = self.input - self.res_quad * self.delay[5];
            self.stage[0] =
                self.delay[0] + self.tune * (f32::tanh(inp * THERMAL) - self.stage_tanh[0]);
            self.delay[0] = self.stage[0];
            for k in 1..4 {
                inp = self.stage[k - 1];
                self.stage_tanh[k - 1] = f32::tanh(inp * THERMAL);
                if k == 3 {
                    self.stage[k] = self.delay[k]
                        + self.tune * (self.stage_tanh[k - 1] - f32::tanh(self.delay[k] * THERMAL));
                } else {
                    self.stage[k] =
                        self.delay[k] + self.tune * (self.stage_tanh[k - 1] - self.stage_tanh[k]);
                }
                self.delay[k] = self.stage[k];
            }
            self.delay[5] = (self.stage[3] + self.delay[4]) * 0.5;
            self.delay[4] = self.stage[3];
        }

        self.output = self.delay[5];
    }
}

/// A biquad filter with configurable coefficients.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `f32` | The input signal. |
/// | `1` | `a0` | `f32` | The `a0` coefficient. |
/// | `2` | `a1` | `f32` | The `a1` coefficient. |
/// | `3` | `a2` | `f32` | The `a2` coefficient. |
/// | `4` | `b1` | `f32` | The `b1` coefficient. |
/// | `5` | `b2` | `f32` | The `b2` coefficient. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `f32` | The output signal. |
#[derive(Clone, Debug, Processor)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", processor_typetag)]
pub struct Biquad {
    #[input]
    input: f32,

    /// The `a0` coefficient.
    #[input]
    pub a0: f32,

    /// The `a1` coefficient.
    #[input]
    pub a1: f32,

    /// The `a2` coefficient.
    #[input]
    pub a2: f32,

    /// The `b1` coefficient.
    #[input]
    pub b1: f32,

    /// The `b2` coefficient.
    #[input]
    pub b2: f32,

    #[output]
    output: f32,

    // input state
    x1: f32,
    x2: f32,

    // output state
    y1: f32,
    y2: f32,
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
            input: 0.0,
            output: 0.0,
        }
    }
}

impl Biquad {
    /// Creates a new `Biquad` filter with the given coefficients.
    pub fn new(a0: f32, a1: f32, a2: f32, b1: f32, b2: f32) -> Self {
        Self {
            a0,
            a1,
            a2,
            b1,
            b2,
            ..Default::default()
        }
    }

    pub fn update(&mut self, _env: &ProcEnv) {
        let filtered = self.a0 * self.input + self.a1 * self.x1 + self.a2 * self.x2
            - self.b1 * self.y1
            - self.b2 * self.y2;

        self.x2 = self.x1;
        self.x1 = self.input;
        self.y2 = self.y1;
        self.y1 = filtered;

        self.output = filtered;
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
#[derive(Clone, Debug, Processor)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", processor_typetag)]
pub struct AutoBiquad {
    // biquad state
    a0: f32,
    a1: f32,
    a2: f32,
    b1: f32,
    b2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,

    // the type of biquad filter
    biquad_type: BiquadType,

    #[input]
    input: f32,

    /// The cutoff frequency of the filter.
    #[input]
    pub cutoff: f32,

    /// The Q factor of the filter.
    #[input]
    pub q: f32,

    /// The gain of the filter.
    #[input]
    pub gain: f32,

    #[output]
    output: f32,
}

impl AutoBiquad {
    /// Creates a new `AutoBiquad` filter with the given type, cutoff frequency, Q factor, and gain.
    pub fn new(biquad_type: BiquadType, cutoff: f32, q: f32, gain: f32) -> Self {
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
            input: 0.0,
            output: 0.0,
        }
    }

    /// Creates a new lowpass `AutoBiquad` filter with the given cutoff frequency and Q factor.
    pub fn lowpass(cutoff: f32, q: f32) -> Self {
        Self::new(BiquadType::LowPass, cutoff, q, 0.0)
    }

    /// Creates a new highpass `AutoBiquad` filter with the given cutoff frequency and Q factor.
    pub fn highpass(cutoff: f32, q: f32) -> Self {
        Self::new(BiquadType::HighPass, cutoff, q, 0.0)
    }

    /// Creates a new bandpass `AutoBiquad` filter with the given cutoff frequency and Q factor.
    pub fn bandpass(cutoff: f32, q: f32) -> Self {
        Self::new(BiquadType::BandPass, cutoff, q, 0.0)
    }

    /// Creates a new notch `AutoBiquad` filter with the given cutoff frequency and Q factor.
    pub fn notch(cutoff: f32, q: f32) -> Self {
        Self::new(BiquadType::Notch, cutoff, q, 0.0)
    }

    /// Creates a new peak `AutoBiquad` filter with the given cutoff frequency, Q factor, and gain.
    pub fn peak(cutoff: f32, q: f32, gain: f32) -> Self {
        Self::new(BiquadType::Peak, cutoff, q, gain)
    }

    /// Creates a new low shelf `AutoBiquad` filter with the given cutoff frequency, Q factor, and gain.
    pub fn low_shelf(cutoff: f32, q: f32, gain: f32) -> Self {
        Self::new(BiquadType::LowShelf, cutoff, q, gain)
    }

    /// Creates a new high shelf `AutoBiquad` filter with the given cutoff frequency, Q factor, and gain.
    pub fn high_shelf(cutoff: f32, q: f32, gain: f32) -> Self {
        Self::new(BiquadType::HighShelf, cutoff, q, gain)
    }

    /// Returns the type of biquad filter this is.
    pub fn biquad_type(&self) -> BiquadType {
        self.biquad_type
    }

    // http://www.earlevel.com/scripts/widgets/20131013/biquads2.js
    #[inline]
    fn set_coefficients(&mut self, sample_rate: f32) {
        if self.q < 0.01 {
            self.q = 0.01;
        }

        let v = f32::powf(10.0, self.gain.abs() / 20.0);
        let k = f32::tan(PI * self.cutoff / sample_rate);

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
                    let norm = 1.0 / (1.0 + f32::sqrt(2.0) * k + k * k);
                    self.a0 = (1.0 + f32::sqrt(2.0 * v) * k + v * k * k) * norm;
                    self.a1 = 2.0 * (v * k * k - 1.0) * norm;
                    self.a2 = (1.0 - f32::sqrt(2.0 * v) * k + v * k * k) * norm;
                    self.b1 = 2.0 * (k * k - 1.0) * norm;
                    self.b2 = (1.0 - f32::sqrt(2.0) * k + k * k) * norm;
                } else {
                    let norm = 1.0 / (1.0 + f32::sqrt(2.0) * k + k * k);
                    self.a0 = (v + f32::sqrt(2.0 * v) * k + k * k) * norm;
                    self.a1 = 2.0 * (k * k - v) * norm;
                    self.a2 = (v - f32::sqrt(2.0 * v) * k + k * k) * norm;
                    self.b1 = 2.0 * (k * k - 1.0) * norm;
                    self.b2 = (1.0 - f32::sqrt(2.0) * k + k * k) * norm;
                }
            }
            BiquadType::HighShelf => {
                if self.gain >= 0.0 {
                    let norm = 1.0 / (1.0 + f32::sqrt(2.0) * k + k * k);
                    self.a0 = (v + f32::sqrt(2.0 * v) * k + k * k) * norm;
                    self.a1 = 2.0 * (k * k - v) * norm;
                    self.a2 = (v - f32::sqrt(2.0 * v) * k + k * k) * norm;
                    self.b1 = 2.0 * (k * k - 1.0) * norm;
                    self.b2 = (1.0 - f32::sqrt(2.0) * k + k * k) * norm;
                } else {
                    let norm = 1.0 / (v + f32::sqrt(2.0 * v) * k + k * k);
                    self.a0 = (1.0 + f32::sqrt(2.0) * k + k * k) * norm;
                    self.a1 = 2.0 * (k * k - 1.0) * norm;
                    self.a2 = (1.0 - f32::sqrt(2.0) * k + k * k) * norm;
                    self.b1 = 2.0 * (v * k * k - 1.0) * norm;
                    self.b2 = (v - f32::sqrt(2.0 * v) * k + v * k * k) * norm;
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

    pub fn update(&mut self, env: &ProcEnv) {
        self.set_coefficients(env.sample_rate);

        let filtered = self.a0 * self.input + self.a1 * self.x1 + self.a2 * self.x2
            - self.b1 * self.y1
            - self.b2 * self.y2;

        self.x2 = self.x1;
        self.x1 = self.input;
        self.y2 = self.y1;
        self.y1 = filtered;

        self.output = filtered;
    }
}

/// A 1-pole lowpass filter.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `f32` | The input signal. |
/// | `1` | `cutoff` | `f32` | The cutoff frequency of the filter. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `f32` | The output signal. |
#[derive(Clone, Debug, Processor)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", processor_typetag)]
pub struct OnePole {
    #[input]
    input: f32,
    #[input]
    cutoff: f32,

    #[output]
    output: f32,

    a0: f32,
    b1: f32,
    x1: f32,
}

impl Default for OnePole {
    fn default() -> Self {
        Self {
            cutoff: 1000.0,
            a0: 1.0,
            b1: 0.0,
            x1: 0.0,
            input: 0.0,
            output: 0.0,
        }
    }
}

impl OnePole {
    /// Creates a new `OnePole` filter with the given cutoff frequency.
    pub fn new(cutoff: f32) -> Self {
        Self {
            cutoff,
            ..Default::default()
        }
    }

    pub fn update(&mut self, env: &ProcEnv) {
        self.cutoff = self.cutoff.clamp(0.0, env.sample_rate * 0.5);
        self.b1 = f32::exp(-2.0 * PI * self.cutoff / env.sample_rate);
        self.a0 = 1.0 - self.b1;

        let filtered = self.a0 * self.input + self.b1 * self.x1;

        self.x1 = self.input;

        self.output = filtered;
    }
}
