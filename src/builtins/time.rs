//! Time-related processors.

use crate::prelude::*;

use super::lerp;

/// A processor that generates a single-sample pulse at regular intervals.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `period` | `f32` | The period of the pulse in seconds. |
/// | `1` | `reset` | `Bool` | Whether to reset the pulse generator. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Bool` | The pulse signal. |
#[derive(Debug, Clone, Processor)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", processor_typetag)]
pub struct Metro {
    #[input]
    period: f32,

    #[input]
    reset: bool,

    #[output]
    out: bool,

    last_time: u64,
    next_time: u64,
    time: u64,
}

impl Metro {
    /// Creates a new `Metro` processor with the given period.
    pub fn new(period: f32) -> Self {
        Self {
            period,
            last_time: 0,
            next_time: 0,
            time: 0,
            reset: false,
            out: false,
        }
    }

    fn next_sample(&mut self, sample_rate: f32) -> bool {
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

    fn update(&mut self, env: &ProcEnv) {
        if self.reset {
            self.time = 0;
            self.last_time = 0;
            self.next_time = 0;
            self.reset = false;
        }

        self.out = self.next_sample(env.sample_rate);
    }
}

impl Default for Metro {
    fn default() -> Self {
        Self::new(1.0)
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
/// | `0` | `in` | `f32` | The input signal. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `f32` | The delayed signal. |
#[derive(Debug, Clone, Default, Processor)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", processor_typetag)]
pub struct UnitDelay {
    value: Option<f32>,

    #[input]
    input: f32,

    #[output]
    out: f32,
}

impl UnitDelay {
    /// Creates a new `UnitDelay` processor.
    pub fn new() -> Self {
        Self::default()
    }

    fn update(&mut self, _env: &ProcEnv) {
        self.out = self.value.unwrap_or_default();
        self.value = Some(self.input);
    }
}

/// A processor that delays a signal by a number of samples.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `f32` | The input signal. |
/// | `1` | `delay` | `Int` | The delay in samples. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `f32` | The delayed signal. |
#[derive(Debug, Clone, Processor)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", processor_typetag)]
pub struct SampleDelay {
    #[cfg_attr(feature = "serde", serde(skip))]
    ring_buffer: Vec<f32>,
    head: usize,

    #[input]
    input: f32,

    #[input]
    delay: i64,

    #[output]
    out: f32,
}

impl SampleDelay {
    /// Creates a new `SampleDelay` processor with the given maximum delay.
    pub fn new(max_delay: usize) -> Self {
        let ring_buffer = vec![0.0; max_delay];
        Self {
            ring_buffer,
            head: 0,
            input: 0.0,
            delay: 0,
            out: 0.0,
        }
    }

    #[inline]
    fn index_modulo(&self, delay: usize) -> usize {
        (self.head + self.ring_buffer.len() - delay) % self.ring_buffer.len()
    }

    fn update(&mut self, _env: &ProcEnv) {
        self.ring_buffer[self.head] = self.input;

        let index = self.index_modulo(self.delay as usize);
        self.out = self.ring_buffer[index];

        self.head = (self.head + 1) % self.ring_buffer.len();
    }
}

/// A processor that delays a signal by a number of samples with linear interpolation.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `f32` | The input signal. |
/// | `1` | `delay` | `f32` | The delay in samples. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `f32` | The delayed signal. |
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FractDelay {
    #[cfg_attr(feature = "serde", serde(skip))]
    ring_buffer: Vec<f32>,
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
    fn index_modulo(&self, delay: f32) -> (usize, f32) {
        let delay_floor = delay.floor() as usize;
        let delay_frac = delay - delay_floor as f32;
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

    fn allocate(&mut self, sample_rate: f32, _max_block_size: usize) {
        self.ring_buffer.resize(sample_rate as usize * 2, 0.0);
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (input, delay, out) in iter_proc_io_as!(
            inputs as [f32, f32],
            outputs as [f32]
        ) {
            let delay = delay.unwrap_or_default();

            self.ring_buffer[self.head] = input.unwrap_or_default();

            let (index, delay_frac) = self.index_modulo(delay);

            let delayed = self.ring_buffer[index];

            let next_index = (index + 1) % self.ring_buffer.len();
            let next = self.ring_buffer[next_index];

            out.set(lerp(delayed, next, delay_frac));

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
/// | `1` | `tau` | `f32` | The decay time constant. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `f32` | The envelope signal. |
#[derive(Debug, Clone, Processor)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", processor_typetag)]
pub struct DecayEnv {
    last_trig: bool,

    value: f32,
    time: f32,

    #[input]
    trig: bool,

    #[input]
    tau: f32,

    #[output]
    out: f32,
}

impl DecayEnv {
    /// Creates a new `DecayEnv` processor with the given decay time constant.
    pub fn new(tau: f32) -> Self {
        Self {
            last_trig: false,
            tau,
            value: 0.0,
            time: 1000.0,
            trig: false,
            out: 0.0,
        }
    }

    fn update(&mut self, env: &ProcEnv) {
        self.tau = self.tau.max(0.0);
        let trig = self.trig;

        if trig && !self.last_trig {
            self.value = 1.0;
            self.time = 0.0;
        } else {
            self.time += env.sample_rate.recip();
            self.value = (-self.tau.recip() * self.time).exp();
        }

        self.last_trig = trig;

        self.value = self.value.clamp(0.0, 1.0);

        self.out = self.value;
    }
}

impl Default for DecayEnv {
    fn default() -> Self {
        Self::new(1.0)
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
/// | `1` | `decay` | `f32` | The decay time in seconds. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `f32` | The envelope signal. |
#[derive(Debug, Clone, Processor)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", processor_typetag)]
pub struct LinearDecayEnv {
    last_trig: bool,

    value: f32,
    time: f32,

    #[input]
    trig: bool,
    #[input]
    decay: f32,

    #[output]
    out: f32,
}

impl LinearDecayEnv {
    /// Creates a new `LinearDecayEnv` processor with the given decay time.
    pub fn new(decay: f32) -> Self {
        Self {
            last_trig: false,
            decay,
            value: 0.0,
            time: 1000.0,
            trig: false,
            out: 0.0,
        }
    }

    fn update(&mut self, env: &ProcEnv) {
        self.decay = self.decay.max(0.0);

        if self.trig && !self.last_trig {
            self.value = 1.0;
            self.time = 0.0;
        } else {
            self.time += env.sample_rate.recip();
            self.value = 1.0 - self.time / self.decay;
        }

        self.last_trig = self.trig;

        self.value = self.value.clamp(0.0, 1.0);

        self.out = self.value;
    }
}

impl Default for LinearDecayEnv {
    fn default() -> Self {
        Self::new(1.0)
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
/// | `1` | `attack` | `f32` | The attack time in seconds. |
/// | `2` | `release` | `f32` | The release time in seconds. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `f32` | The envelope signal. |
#[derive(Debug, Clone, Processor)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", processor_typetag)]
pub struct AREnv {
    last_trig: bool,
    value: f32,
    state: ADSRState,

    #[input]
    trig: bool,
    #[input]
    attack: f32,
    #[input]
    release: f32,

    #[output]
    out: f32,
}

impl AREnv {
    /// Creates a new `AREnv` processor with the given attack and release times.
    pub fn new(attack: f32, release: f32) -> Self {
        Self {
            last_trig: false,
            attack,
            release,
            value: 0.0,
            state: ADSRState::Sustain,
            trig: false,
            out: 0.0,
        }
    }

    fn update(&mut self, env: &ProcEnv) {
        self.attack = self.attack.max(0.0);
        self.release = self.release.max(0.0);

        if self.trig && !self.last_trig {
            self.value = 0.0;
            self.state = ADSRState::Attack;
        } else if !self.trig && self.last_trig {
            self.state = ADSRState::Release;
        }

        let slope = match self.state {
            ADSRState::Sustain => 0.0,
            ADSRState::Attack => 1.0 / (self.attack * env.sample_rate),
            ADSRState::Release => -1.0 / (self.release * env.sample_rate),
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

        self.last_trig = self.trig;

        self.out = self.value;
    }
}

impl Default for AREnv {
    fn default() -> Self {
        Self::new(0.0, 0.0)
    }
}

/// A linear ADSR (attack-decay-sustain-release) envelope generator.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `gate` | `Bool` | The gate signal. |
/// | `1` | `attack` | `f32` | The attack time in seconds. |
/// | `2` | `decay` | `f32` | The decay time in seconds. |
/// | `3` | `sustain` | `f32` | The sustain level. |
/// | `4` | `release` | `f32` | The release time in seconds. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `f32` | The envelope signal. |
#[derive(Debug, Clone, Processor)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", processor_typetag)]
pub struct ADSREnv {
    last_trig: bool,
    value: f32,
    state: ADSRState,

    #[input]
    trig: bool,

    #[input]
    attack: f32,
    #[input]
    decay: f32,
    #[input]
    sustain: f32,
    #[input]
    release: f32,

    #[output]
    out: f32,
}

impl ADSREnv {
    /// Creates a new `ADSREnv` processor with the given attack, decay, sustain, and release times.
    pub fn new(attack: f32, decay: f32, sustain: f32, release: f32) -> Self {
        Self {
            last_trig: false,
            attack,
            decay,
            sustain,
            release,
            value: 0.0,
            state: ADSRState::Sustain,
            trig: false,
            out: 0.0,
        }
    }

    fn update(&mut self, env: &ProcEnv) {
        self.attack = self.attack.max(0.0);
        self.decay = self.decay.max(0.0);
        self.sustain = self.sustain.max(0.0);
        self.release = self.release.max(0.0);

        if self.trig && !self.last_trig {
            self.value = 0.0;
            self.state = ADSRState::Attack;
        } else if !self.trig && self.last_trig {
            self.state = ADSRState::Release;
        }

        let slope = match self.state {
            ADSRState::Attack => 1.0 / (self.attack * env.sample_rate),
            ADSRState::Decay => -(1.0 - self.sustain) / (self.decay * env.sample_rate),
            ADSRState::Sustain => 0.0,
            ADSRState::Release => -self.sustain / (self.release * env.sample_rate),
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

        self.last_trig = self.trig;

        self.out = self.value;
    }
}

impl Default for ADSREnv {
    fn default() -> Self {
        Self::new(0.0, 0.0, 1.0, 0.0)
    }
}
