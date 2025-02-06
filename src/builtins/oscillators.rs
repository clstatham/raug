//! Oscillator processors.

use std::collections::VecDeque;

use crate::prelude::*;

/// A processor that accumulates a phase value.
///
/// The phase value will be incremented by the `increment` input signal each sample, and can be reset to 0 by the `reset` input signal.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `increment` | `Float` | The phase increment per sample. |
/// | `1` | `reset` | `Bool` | Whether to reset the phase accumulator to 0. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Float` | The phase accumulator value. |
#[derive(Clone, Debug, Default, Processor)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", processor_typetag)]
pub struct PhaseAccumulator {
    // phase accumulator
    t: Float,
    // phase increment per sample
    #[input]
    increment: Float,
    #[input]
    reset: bool,

    #[output]
    out: Float,
}

impl PhaseAccumulator {
    pub fn update(&mut self, _env: &ProcEnv) {
        // increment the phase accumulator
        self.t += self.increment;

        // check for phase reset
        if self.reset {
            self.t = 0.0;
        }

        self.out = self.t;
    }
}

/// A processor that generates a sine wave.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `frequency` | `Float` | The frequency of the sine wave. |
/// | `1` | `phase` | `Float` | The phase offset of the sine wave. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Float` | The sine wave value. |
#[derive(Clone, Debug, Processor)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", processor_typetag)]
pub struct SineOscillator {
    // phase accumulator
    t: Float,
    // phase increment per sample
    t_step: Float,

    /// The frequency of the sine wave.
    #[input]
    pub frequency: Float,

    /// The phase offset of the sine wave.
    #[input]
    pub phase: Float,

    #[input]
    reset: bool,

    #[output]
    out: Float,
}

impl SineOscillator {
    /// Creates a new [`SineOscillator`] processor with the given frequency.
    pub fn new(frequency: Float) -> Self {
        Self {
            frequency,
            ..Default::default()
        }
    }

    pub fn update(&mut self, env: &ProcEnv) {
        // calculate the sine wave using the phase accumulator
        self.out = (self.t / env.sample_rate * TAU + self.phase).cos();

        // increment the phase accumulator
        self.t_step = self.frequency;
        self.t += self.t_step;
        self.t %= env.sample_rate;
    }
}

impl Default for SineOscillator {
    fn default() -> Self {
        Self {
            t: 0.0,
            t_step: 0.0,
            frequency: 0.0,
            phase: 0.0,
            out: 0.0,
            reset: false,
        }
    }
}

/// A processor that generates a unipolar sawtooth wave, appropriate for use as a modulation source.
///
/// This processor's output is not anti-aliased. For band-limited sawtooth waves, see the [`BlSawOscillator`] processor.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `frequency` | `Float` | The frequency of the sawtooth wave. |
/// | `1` | `phase` | `Float` | The phase offset of the sawtooth wave. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Float` | The sawtooth wave value. |
#[derive(Clone, Debug, Processor)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", processor_typetag)]
pub struct SawOscillator {
    // phase accumulator
    t: Float,
    // phase increment per sample
    t_step: Float,

    /// The frequency of the sawtooth wave.
    #[input]
    pub frequency: Float,

    /// The phase offset of the sawtooth wave.
    #[input]
    pub phase: Float,

    #[input]
    reset: bool,

    #[output]
    out: Float,
}

impl Default for SawOscillator {
    fn default() -> Self {
        Self {
            t: 0.0,
            t_step: 0.0,
            frequency: 0.0,
            phase: 0.0,
            out: 0.0,
            reset: false,
        }
    }
}

impl SawOscillator {
    /// Creates a new [`SawOscillator`] processor with the given frequency.
    pub fn new(frequency: Float) -> Self {
        Self {
            frequency,
            ..Default::default()
        }
    }

    pub fn update(&mut self, env: &ProcEnv) {
        // calculate the sawtooth wave using the phase accumulator
        self.out = (self.t / env.sample_rate + self.phase) % 1.0;

        // increment the phase accumulator
        self.t_step = self.frequency;
        self.t += self.t_step;
        self.t %= env.sample_rate;

        if self.reset {
            self.t = 0.0;
        }
    }
}

/// A processor that generates unipolar white noise.
///
/// # Inputs
///
/// None.
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Float` | The white noise value. |
#[derive(Clone, Debug, Processor)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", processor_typetag)]
pub struct NoiseOscillator {
    #[output]
    out: Float,
}

impl NoiseOscillator {
    /// Creates a new [`NoiseOscillator`] processor.
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, _env: &ProcEnv) {
        self.out = rand::random::<Float>();
    }
}

impl Default for NoiseOscillator {
    fn default() -> Self {
        Self { out: 0.0 }
    }
}

/// A processor that generates a band-limited sawtooth wave.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `frequency` | `Float` | The frequency of the sawtooth wave. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Float` | The sawtooth wave value. |
#[derive(Clone, Debug, Processor)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", processor_typetag)]
pub struct BlSawOscillator {
    p: Float,
    dp: Float,
    saw: Float,

    /// The frequency of the sawtooth wave.
    #[input]
    pub frequency: Float,

    #[output]
    out: Float,
}

impl Default for BlSawOscillator {
    fn default() -> Self {
        Self {
            p: 0.0,
            dp: 1.0,
            saw: 0.0,
            frequency: 0.0,
            out: 0.0,
        }
    }
}

impl BlSawOscillator {
    /// Creates a new [`BlSawOscillator`] processor with the given frequency.
    pub fn new(frequency: Float) -> Self {
        Self {
            frequency,
            ..Default::default()
        }
    }

    pub fn update(&mut self, env: &ProcEnv) {
        // algorithm courtesy of https://www.musicdsp.org/en/latest/Synthesis/12-bandlimited-waveforms.html
        if self.frequency <= 0.0 {
            self.out = 0.0;
            return;
        }

        let pmax = 0.5 * env.sample_rate / self.frequency;
        let dc = -0.498 / pmax;

        self.p += self.dp;
        if self.p < 0.0 {
            self.p = -self.p;
            self.dp = -self.dp;
        } else if self.p > pmax {
            self.p = 2.0 * pmax - self.p;
            self.dp = -self.dp;
        }

        let mut x = PI * self.p;
        if x < 0.00001 {
            x = 0.00001;
        }

        self.saw = 0.995 * self.saw + dc + x.sin() / x;

        self.out = self.saw;
    }
}

const BL_SQUARE_MAX_HARMONICS: usize = 512;

/// A processor that generates a band-limited square/pulse wave.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `frequency` | `Float` | The frequency of the square wave. |
/// | `1` | `pulse_width` | `Float` | The pulse width of the square wave. |
/// | `2` | `reset` | `Bool` | Whether to reset the phase accumulator to 0. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Float` | The square wave value. |
#[derive(Clone, Debug, Processor)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", processor_typetag)]
pub struct BlSquareOscillator {
    // phase accumulator
    t: Float,
    // phase increment per sample
    t_step: Float,

    // band-limited square wave coefficients
    coeff: Box<[Float]>,

    /// The frequency of the square wave.
    #[input]
    pub frequency: Float,

    /// The pulse width of the square wave (0.0 to 1.0).
    #[input]
    pub pulse_width: Float,

    #[input]
    reset: bool,

    #[output]
    out: Float,
}

impl Default for BlSquareOscillator {
    fn default() -> Self {
        Self::new(0.0, 0.5)
    }
}

impl BlSquareOscillator {
    /// Creates a new [`BlSquareOscillator`] processor with the given frequency and pulse width.
    pub fn new(frequency: Float, pulse_width: Float) -> Self {
        Self {
            frequency,
            pulse_width,
            t: 0.0,
            t_step: 0.0,
            coeff: Box::new([0.0; BL_SQUARE_MAX_HARMONICS]),
            out: 0.0,
            reset: false,
        }
    }

    pub fn update(&mut self, env: &ProcEnv) {
        self.frequency = self.frequency.max(0.0);
        if self.frequency <= 0.0 {
            self.out = 0.0;
            return;
        }

        if self.reset {
            self.t = 0.0;
        }

        self.pulse_width = self.pulse_width.clamp(0.0, 1.0);

        self.t_step = self.frequency / env.sample_rate;

        let n_harm = (env.sample_rate / (self.frequency * 4.0)) as usize;
        self.coeff[0] = self.pulse_width - 0.5;
        for i in 1..n_harm + 1 {
            self.coeff[i] =
                Float::sin(i as Float * PI * self.pulse_width) * 2.0 / (i as Float * PI);
        }

        let theta = self.t * TAU;

        let mut square = 0.0;
        for i in 0..n_harm + 1 {
            square += self.coeff[i] * (theta * i as Float).cos();
        }

        self.t += self.t_step;
        self.out = square;
    }
}

/// A processor that models a physical string vibrating at a given frequency using the Karplus-Strong algorithm.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `trig` | `bool` | Triggers the pluck. |
/// | `1` | `frequency` | `Float` | The frequency of the string. |
/// | `2` | `damping` | `Float` | The damping factor of the string. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Float` | The string value. |
#[derive(Clone, Debug, Processor)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", processor_typetag)]
pub struct KarplusStrong {
    // delay line
    ringbuf: VecDeque<Float>,

    #[input]
    trig: bool,

    /// The frequency of the string.
    #[input]
    pub frequency: Float,

    /// The damping factor of the string.
    #[input]
    pub damping: Float,

    #[output]
    out: Float,
}

impl KarplusStrong {
    /// Creates a new [`KarplusStrong`] processor with the given frequency, damping factor, and pluck position.
    pub fn new(frequency: Float, damping: Float) -> Self {
        Self {
            ringbuf: VecDeque::new(),
            damping,
            frequency,
            trig: false,
            out: 0.0,
        }
    }

    pub fn update(&mut self, env: &ProcEnv) {
        self.frequency = self.frequency.max(0.0);
        if self.frequency <= 0.0 {
            self.out = 0.0;
            return;
        }

        if self.trig {
            // calculate the delay line index
            let delay_time = (env.sample_rate / self.frequency) as usize;

            // initialize the delay line with noise
            self.ringbuf.clear();
            for _ in 0..delay_time {
                self.ringbuf.push_back(rand::random::<Float>() * 2.0 - 1.0);
            }
        }

        let first = self.ringbuf.pop_front().unwrap_or_default();
        let second = *self.ringbuf.front().unwrap_or(&0.0);

        let new_sample = (first + second) * 0.5 * (1.0 - self.damping) + first * self.damping;

        self.ringbuf.push_back(new_sample);

        self.out = first;
    }
}

impl Default for KarplusStrong {
    fn default() -> Self {
        Self::new(0.0, 0.5)
    }
}
