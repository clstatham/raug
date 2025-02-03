//! Time-related processors.

use crate::prelude::*;

use super::lerp;

/// A processor that generates a single-sample pulse at regular intervals.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `period` | `Float` | The period of the pulse in seconds. |
/// | `1` | `reset` | `Bool` | Whether to reset the pulse generator. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Bool` | The pulse signal. |
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Metro {
    period: Float,
    last_time: u64,
    next_time: u64,
    time: u64,
}

impl Metro {
    /// Creates a new `Metro` processor with the given period.
    pub fn new(period: Float) -> Self {
        Self {
            period,
            last_time: 0,
            next_time: 0,
            time: 0,
        }
    }

    fn next_sample(&mut self, sample_rate: Float) -> bool {
        let out = if self.time >= self.next_time {
            self.last_time = self.time;
            self.next_time = self.time + (self.period * sample_rate) as u64;
            true
        } else {
            false
        };

        self.time += 1;

        out
    }
}

impl Default for Metro {
    fn default() -> Self {
        Self::new(1.0)
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for Metro {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("period", SignalType::Float),
            SignalSpec::new("reset", SignalType::Bool),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalType::Bool)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (period, reset, out) in iter_proc_io!(
            inputs as [Float, bool],
            outputs as [bool]
        ) {
            if reset.unwrap_or(false) {
                self.time = 0;
                self.last_time = 0;
                self.next_time = 0;
            }

            self.period = period.unwrap_or(self.period);

            if self.next_sample(inputs.sample_rate()) {
                *out = Some(true);
            } else {
                *out = None;
            }
        }

        Ok(())
    }
}

/// A processor that delays a signal by one sample.
///
/// Note that feedback loops in a [`Graph`] implicitly introduce a delay of one sample, so this processor is not usually required to be used manually.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `Float` | The input signal. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Float` | The delayed signal. |
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UnitDelay {
    value: Option<Float>,
}

impl UnitDelay {
    /// Creates a new `UnitDelay` processor.
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for UnitDelay {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("in", SignalType::Float)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalType::Float)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (in_signal, out) in iter_proc_io!(inputs as [Float], outputs as [Float]) {
            *out = self.value;
            self.value = *in_signal;
        }

        Ok(())
    }
}

/// A processor that delays a signal by a number of samples.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `Float` | The input signal. |
/// | `1` | `delay` | `Int` | The delay in samples. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Float` | The delayed signal. |
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SampleDelay {
    ring_buffer: Vec<Float>,
    head: usize,
}

impl SampleDelay {
    /// Creates a new `SampleDelay` processor with the given maximum delay.
    pub fn new(max_delay: usize) -> Self {
        let ring_buffer = vec![0.0; max_delay];
        Self {
            ring_buffer,
            head: 0,
        }
    }

    #[inline]
    fn index_modulo(&self, delay: usize) -> usize {
        (self.head + self.ring_buffer.len() - delay) % self.ring_buffer.len()
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for SampleDelay {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("in", SignalType::Float),
            SignalSpec::new("delay", SignalType::Int),
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
        for (in_signal, delay, out) in iter_proc_io!(
            inputs as [Float, i64],
            outputs as [Float]
        ) {
            let in_signal = in_signal.unwrap_or_default();

            let delay = delay.unwrap_or_default() as usize;

            self.ring_buffer[self.head] = in_signal;

            let index = self.index_modulo(delay);
            *out = Some(self.ring_buffer[index]);

            self.head = (self.head + 1) % self.ring_buffer.len();
        }

        Ok(())
    }
}

/// A processor that delays a signal by a number of samples with linear interpolation.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `Float` | The input signal. |
/// | `1` | `delay` | `Float` | The delay in samples. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Float` | The delayed signal. |
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FractDelay {
    #[cfg_attr(feature = "serde", serde(skip))]
    ring_buffer: Vec<Float>,
    head: usize,
}

impl FractDelay {
    /// Creates a new `FractDelay` processor with the given maximum delay.
    pub fn new() -> Self {
        Self {
            ring_buffer: vec![0.0; 2],
            head: 0,
        }
    }

    #[inline]
    fn index_modulo(&self, delay: Float) -> (usize, Float) {
        let delay_floor = delay.floor() as usize;
        let delay_frac = delay - delay_floor as Float;
        let index = (self.head + self.ring_buffer.len() - delay_floor) % self.ring_buffer.len();
        (index, delay_frac)
    }
}

impl Default for FractDelay {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for FractDelay {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("in", SignalType::Float),
            SignalSpec::new("delay", SignalType::Float),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalType::Float)]
    }

    fn allocate(&mut self, sample_rate: Float, _max_block_size: usize) {
        self.ring_buffer.resize(sample_rate as usize * 2, 0.0);
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (in_signal, delay, out) in iter_proc_io!(
            inputs as [Float, Float],
            outputs as [Float]
        ) {
            let delay = delay.unwrap_or_default();

            self.ring_buffer[self.head] = in_signal.unwrap_or_default();

            let (index, delay_frac) = self.index_modulo(delay);

            let delayed = self.ring_buffer[index];

            let next_index = (index + 1) % self.ring_buffer.len();
            let next = self.ring_buffer[next_index];

            *out = Some(lerp(delayed, next, delay_frac));

            self.head = (self.head + 1) % self.ring_buffer.len();
        }

        Ok(())
    }
}

/// A processor that generates an exponential decay envelope signal.
///
/// The envelope is generated by the following formula:
///
/// ```text
/// y(t) = exp(-t / tau)
/// ```
///
/// where `t` is the time since the last trigger and `tau` is the decay time constant.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `trig` | `Bool` | The trigger signal. |
/// | `1` | `tau` | `Float` | The decay time constant. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Float` | The envelope signal. |
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DecayEnv {
    last_trig: bool,
    tau: Float,
    value: Float,
    time: Float,
}

impl DecayEnv {
    /// Creates a new `DecayEnv` processor with the given decay time constant.
    pub fn new(tau: Float) -> Self {
        Self {
            last_trig: false,
            tau,
            value: 0.0,
            time: 1000.0,
        }
    }
}

impl Default for DecayEnv {
    fn default() -> Self {
        Self::new(1.0)
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for DecayEnv {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("trig", SignalType::Bool),
            SignalSpec::new("tau", SignalType::Float),
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
        for (trig, tau, out) in iter_proc_io!(
            inputs as [bool, Float],
            outputs as [Float]
        ) {
            self.tau = tau.unwrap_or(self.tau);
            let trig = trig.unwrap_or(false);

            if trig && !self.last_trig {
                self.value = 1.0;
                self.time = 0.0;
            } else {
                self.time += inputs.sample_rate().recip();
                self.value = (-self.tau.recip() * self.time).exp();
            }

            self.last_trig = trig;

            self.value = self.value.clamp(0.0, 1.0);

            *out = Some(self.value);
        }

        Ok(())
    }
}

/// A processor that generates a linear decay envelope signal.
///
/// The envelope is generated by the following formula:
///
/// ```text
/// y(t) = 1 - t / decay
/// ```
///
/// where `t` is the time since the last trigger in seconds and `decay` is the decay time in seconds.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `trig` | `Bool` | The trigger signal. |
/// | `1` | `decay` | `Float` | The decay time in seconds. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Float` | The envelope signal. |
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LinearDecayEnv {
    last_trig: bool,
    decay: Float,
    value: Float,
    time: Float,
}

impl LinearDecayEnv {
    /// Creates a new `LinearDecayEnv` processor with the given decay time.
    pub fn new(decay: Float) -> Self {
        Self {
            last_trig: false,
            decay,
            value: 0.0,
            time: 1000.0,
        }
    }
}

impl Default for LinearDecayEnv {
    fn default() -> Self {
        Self::new(1.0)
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for LinearDecayEnv {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("trig", SignalType::Bool),
            SignalSpec::new("decay", SignalType::Float),
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
        for (trig, decay, out) in iter_proc_io!(
            inputs as [bool, Float],
            outputs as [Float]
        ) {
            self.decay = decay.unwrap_or(self.decay);
            if self.decay < 0.0 {
                return Err(ProcessorError::InvalidValue("decay time msut be positive"));
            }
            let trig = trig.unwrap_or(false);

            if trig && !self.last_trig {
                self.value = 1.0;
                self.time = 0.0;
            } else {
                self.time += inputs.sample_rate().recip();
                self.value = 1.0 - self.time / self.decay;
            }

            self.last_trig = trig;

            self.value = self.value.clamp(0.0, 1.0);

            *out = Some(self.value);
        }

        Ok(())
    }
}

/// The state of an ADSR envelope generator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ADSRState {
    /// The envelope is ramping up to 1.0.
    Attack,
    /// The envelope is ramping down to the sustain level.
    Decay,
    /// The envelope is sustaining its current level.
    Sustain,
    /// The envelope is ramping down to 0.0.
    Release,
}

/// A linear AR (attack-release) envelope generator.
///
/// The envelope will ramp up to 1.0 when the gate goes high, and ramp down to 0.0 when the gate goes low.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `gate` | `Bool` | The gate signal. |
/// | `1` | `attack` | `Float` | The attack time in seconds. |
/// | `2` | `release` | `Float` | The release time in seconds. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Float` | The envelope signal. |
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AREnv {
    last_trig: bool,
    attack: Float,
    release: Float,
    value: Float,
    state: ADSRState,
}

impl AREnv {
    /// Creates a new `AREnv` processor with the given attack and release times.
    pub fn new(attack: Float, release: Float) -> Self {
        Self {
            last_trig: false,
            attack,
            release,
            value: 0.0,
            state: ADSRState::Sustain,
        }
    }
}

impl Default for AREnv {
    fn default() -> Self {
        Self::new(0.0, 0.0)
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for AREnv {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("gate", SignalType::Bool),
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
        for (trig, attack, release, out) in iter_proc_io!(
            inputs as [bool, Float, Float],
            outputs as [Float]
        ) {
            self.attack = attack.unwrap_or(self.attack);
            self.release = release.unwrap_or(self.release);
            let trig = trig.unwrap_or(false);

            if trig && !self.last_trig {
                self.value = 0.0;
                self.state = ADSRState::Attack; // attack
            } else if !trig && self.last_trig {
                self.state = ADSRState::Release; // release
            }

            let slope = match self.state {
                ADSRState::Sustain => 0.0,
                ADSRState::Attack => 1.0 / (self.attack * inputs.sample_rate()),
                ADSRState::Release => -1.0 / (self.release * inputs.sample_rate()),
                _ => unreachable!(),
            };

            self.value += slope;

            if self.state == ADSRState::Attack && self.value >= 1.0 {
                self.value = 1.0;
                self.state = ADSRState::Sustain;
            } else if self.state == ADSRState::Release && self.value <= 0.0 {
                self.value = 0.0;
                self.state = ADSRState::Sustain;
            }

            self.last_trig = trig;

            *out = Some(self.value);
        }

        Ok(())
    }
}

/// A linear ADSR (attack-decay-sustain-release) envelope generator.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `gate` | `Bool` | The gate signal. |
/// | `1` | `attack` | `Float` | The attack time in seconds. |
/// | `2` | `decay` | `Float` | The decay time in seconds. |
/// | `3` | `sustain` | `Float` | The sustain level. |
/// | `4` | `release` | `Float` | The release time in seconds. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Float` | The envelope signal. |
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ADSREnv {
    last_trig: bool,
    attack: Float,
    decay: Float,
    sustain: Float,
    release: Float,
    value: Float,
    state: ADSRState,
}

impl ADSREnv {
    /// Creates a new `ADSREnv` processor with the given attack, decay, sustain, and release times.
    pub fn new(attack: Float, decay: Float, sustain: Float, release: Float) -> Self {
        Self {
            last_trig: false,
            attack,
            decay,
            sustain,
            release,
            value: 0.0,
            state: ADSRState::Sustain,
        }
    }
}

impl Default for ADSREnv {
    fn default() -> Self {
        Self::new(0.0, 0.0, 1.0, 0.0)
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for ADSREnv {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("gate", SignalType::Bool),
            SignalSpec::new("attack", SignalType::Float),
            SignalSpec::new("decay", SignalType::Float),
            SignalSpec::new("sustain", SignalType::Float),
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
        for (trig, attack, decay, sustain, release, out) in iter_proc_io!(
            inputs as [bool, Float, Float, Float, Float],
            outputs as [Float]
        ) {
            self.attack = attack.unwrap_or(self.attack);
            self.decay = decay.unwrap_or(self.decay);
            self.sustain = sustain.unwrap_or(self.sustain);
            self.release = release.unwrap_or(self.release);
            let trig = trig.unwrap_or(false);

            if trig && !self.last_trig {
                self.value = 0.0;
                self.state = ADSRState::Attack;
            } else if !trig && self.last_trig {
                self.state = ADSRState::Release;
            }

            let slope = match self.state {
                ADSRState::Attack => 1.0 / (self.attack * inputs.sample_rate()),
                ADSRState::Decay => -(1.0 - self.sustain) / (self.decay * inputs.sample_rate()),
                ADSRState::Sustain => 0.0,
                ADSRState::Release => -self.sustain / (self.release * inputs.sample_rate()),
            };

            self.value += slope;

            if self.state == ADSRState::Attack && self.value >= 1.0 {
                self.value = 1.0;
                self.state = ADSRState::Decay;
            } else if self.state == ADSRState::Decay && self.value <= self.sustain {
                self.value = self.sustain;
                self.state = ADSRState::Sustain;
            } else if self.state == ADSRState::Release && self.value <= 0.0 {
                self.value = 0.0;
                self.state = ADSRState::Sustain;
            }

            self.last_trig = trig;

            *out = Some(self.value);
        }

        Ok(())
    }
}
