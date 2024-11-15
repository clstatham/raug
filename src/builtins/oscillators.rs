//! Oscillator processors.

use rand::prelude::Distribution;

use crate::{
    prelude::*,
    processor::ProcessorOutputs,
    signal::{PI, TAU},
};

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
#[derive(Clone, Debug, Default)]
pub struct PhaseAccumulator {
    // phase accumulator
    t: Float,
    // phase increment per sample
    t_step: Float,
}

impl Processor for PhaseAccumulator {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("increment", SignalType::Float),
            SignalSpec::new("reset", SignalType::Bool),
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
        for (out, increment, reset) in itertools::izip!(
            outputs.iter_output_mut_as_floats(0)?,
            inputs.iter_input_as_floats(0)?,
            inputs.iter_input_as_bools(1)?
        ) {
            if reset.unwrap_or(false) {
                self.t = 0.0;
            }

            // output the phase accumulator value
            *out = Some(self.t);

            // increment the phase accumulator
            if let Some(increment) = increment {
                self.t_step = increment;
            }
            self.t += self.t_step;
        }

        Ok(())
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
#[derive(Clone, Debug)]
pub struct SineOscillator {
    // phase accumulator
    t: Float,
    // phase increment per sample
    t_step: Float,
    // sample rate
    sample_rate: Float,

    /// The frequency of the sine wave.
    pub frequency: Float,

    /// The phase offset of the sine wave.
    pub phase: Float,
}

impl SineOscillator {
    /// Creates a new [`SineOscillator`] processor with the given frequency.
    pub fn new(frequency: Float) -> Self {
        Self {
            frequency,
            ..Default::default()
        }
    }
}

impl Default for SineOscillator {
    fn default() -> Self {
        Self {
            t: 0.0,
            t_step: 0.0,
            sample_rate: 0.0,
            frequency: 0.0,
            phase: 0.0,
        }
    }
}

impl Processor for SineOscillator {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("frequency", SignalType::Float),
            SignalSpec::new("phase", SignalType::Float),
            SignalSpec::new("reset", SignalType::Bool),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalType::Float)]
    }

    fn resize_buffers(&mut self, sample_rate: Float, _block_size: usize) {
        self.sample_rate = sample_rate;
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (out, frequency, phase, reset) in itertools::izip!(
            outputs.iter_output_mut_as_floats(0)?,
            inputs.iter_input_as_floats(0)?,
            inputs.iter_input_as_floats(1)?,
            inputs.iter_input_as_bools(2)?
        ) {
            if let Some(true) = reset {
                self.t = 0.0;
            }

            if let Some(frequency) = frequency {
                self.frequency = frequency;
            }

            if let Some(phase) = phase {
                self.phase = phase;
            }

            // calculate the sine wave using the phase accumulator
            let sine = (self.t / self.sample_rate * TAU + self.phase).cos();
            *out = Some(sine);

            // increment the phase accumulator
            self.t_step = self.frequency;
            self.t += self.t_step;
            self.t %= self.sample_rate;
        }

        Ok(())
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
#[derive(Clone, Debug)]
pub struct SawOscillator {
    // phase accumulator
    t: Float,
    // phase increment per sample
    t_step: Float,
    // sample rate
    sample_rate: Float,

    /// The frequency of the sawtooth wave.
    pub frequency: Float,

    /// The phase offset of the sawtooth wave.
    pub phase: Float,
}

impl Default for SawOscillator {
    fn default() -> Self {
        Self {
            t: 0.0,
            t_step: 0.0,
            sample_rate: 0.0,
            frequency: 0.0,
            phase: 0.0,
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
}

impl Processor for SawOscillator {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("frequency", SignalType::Float),
            SignalSpec::new("phase", SignalType::Float),
            SignalSpec::new("reset", SignalType::Bool),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalType::Float)]
    }

    fn resize_buffers(&mut self, sample_rate: Float, _block_size: usize) {
        self.sample_rate = sample_rate;
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (out, frequency, phase, reset) in itertools::izip!(
            outputs.iter_output_mut_as_floats(0)?,
            inputs.iter_input_as_floats(0)?,
            inputs.iter_input_as_floats(1)?,
            inputs.iter_input_as_bools(2)?
        ) {
            if let Some(true) = reset {
                self.t = 0.0;
            }

            if let Some(frequency) = frequency {
                self.frequency = frequency;
            }

            if let Some(phase) = phase {
                self.phase = phase;
            }

            // calculate the sawtooth wave using the phase accumulator
            *out = Some((self.t / self.sample_rate + self.phase) % 1.0);

            // increment the phase accumulator
            self.t_step = self.frequency;
            self.t += self.t_step;
            self.t %= self.sample_rate;
        }

        Ok(())
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
#[derive(Clone, Debug)]
pub struct NoiseOscillator {
    distribution: rand::distributions::Uniform<f64>,
}

impl NoiseOscillator {
    /// Creates a new [`NoiseOscillator`] processor.
    pub fn new() -> Self {
        NoiseOscillator {
            distribution: rand::distributions::Uniform::new(0.0, 1.0),
        }
    }
}

impl Default for NoiseOscillator {
    fn default() -> Self {
        Self::new()
    }
}

impl Processor for NoiseOscillator {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalType::Float)]
    }

    fn process(
        &mut self,
        _inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        let mut rng = rand::thread_rng();
        for out in outputs.iter_output_mut_as_floats(0)? {
            // generate a random number
            *out = Some(self.distribution.sample(&mut rng) as Float);
        }

        Ok(())
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
#[derive(Clone, Debug)]
pub struct BlSawOscillator {
    p: Float,
    dp: Float,
    saw: Float,
    sample_rate: Float,

    /// The frequency of the sawtooth wave.
    pub frequency: Float,
}

impl Default for BlSawOscillator {
    fn default() -> Self {
        Self {
            p: 0.0,
            dp: 1.0,
            saw: 0.0,
            sample_rate: 0.0,
            frequency: 0.0,
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
}

impl Processor for BlSawOscillator {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("frequency", SignalType::Float)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalType::Float)]
    }

    fn resize_buffers(&mut self, sample_rate: Float, _block_size: usize) {
        self.sample_rate = sample_rate;
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        // algorithm courtesy of https://www.musicdsp.org/en/latest/Synthesis/12-bandlimited-waveforms.html
        for (out, frequency) in itertools::izip!(
            outputs.iter_output_mut_as_floats(0)?,
            inputs.iter_input_as_floats(0)?
        ) {
            self.frequency = frequency.unwrap_or(self.frequency);
            if self.frequency <= 0.0 {
                *out = None;
                continue;
            }

            let pmax = 0.5 * self.sample_rate / self.frequency;
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

            *out = Some(self.saw);
        }

        Ok(())
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
#[derive(Clone, Debug)]
pub struct BlSquareOscillator {
    sample_rate: Float,

    // phase accumulator
    t: Float,
    // phase increment per sample
    t_step: Float,

    // band-limited square wave coefficients
    coeff: Box<[Float; BL_SQUARE_MAX_HARMONICS]>,

    /// The frequency of the square wave.
    pub frequency: Float,

    /// The pulse width of the square wave (0.0 to 1.0).
    pub pulse_width: Float,
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
            sample_rate: 0.0,
        }
    }
}

impl Processor for BlSquareOscillator {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("frequency", SignalType::Float),
            SignalSpec::new("pulse_width", SignalType::Float),
            SignalSpec::new("reset", SignalType::Bool),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalType::Float)]
    }

    fn resize_buffers(&mut self, sample_rate: Float, _block_size: usize) {
        self.sample_rate = sample_rate;
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (out, frequency, pulse_width, reset) in itertools::izip!(
            outputs.iter_output_mut_as_floats(0)?,
            inputs.iter_input_as_floats(0)?,
            inputs.iter_input_as_floats(1)?,
            inputs.iter_input_as_bools(2)?
        ) {
            self.frequency = frequency.unwrap_or(self.frequency);
            if self.frequency <= 0.0 {
                *out = None;
                continue;
            }

            if reset.unwrap_or(false) {
                self.t = 0.0;
            }

            self.pulse_width = pulse_width.unwrap_or(self.pulse_width);

            self.t_step = self.frequency / self.sample_rate;

            let n_harm = (self.sample_rate / (self.frequency * 4.0)) as usize;
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

            *out = Some(square);
        }

        Ok(())
    }
}
