use raug::prelude::*;

use super::SineOscillator;

#[processor(derive(Default))]
pub fn metro(
    env: ProcEnv,
    #[state] last_time: &mut f32,
    #[state] next_time: &mut f32,
    #[state] time: &mut f32,
    #[input] period: &f32,
    #[input] reset: &bool,
    #[output] out: &mut bool,
) -> ProcResult<()> {
    if *reset {
        *last_time = 0.0;
        *next_time = 0.0;
        *time = 0.0;
    }

    *out = if *time >= *next_time {
        *last_time = *time;
        *next_time = *time + (*period * env.sample_rate);
        true
    } else {
        false
    };

    *time += 1.0;

    Ok(())
}

impl Metro {
    /// Constructs a new [`Metro`] processor with the given period between ticks.
    pub fn new(period: f32) -> Self {
        Self {
            last_time: 0.0,
            next_time: 0.0,
            time: 0.0,
            period,
            reset: false,
        }
    }

    /// Constructs a new [`Metro`] processor from the given tempo (in BPM, or beats per minute) and TPB (number of ticks per beat).
    /// The metronome will tick `bpm * tpb` times per minute.
    pub fn from_tempo_and_ticks(bpm: f32, tpb: usize) -> Self {
        assert_ne!(tpb, 0, "Ticks per beat mut be >= 1");
        Self::new(60.0 / bpm / (tpb as f32))
    }
}

#[processor(derive(Default))]
pub fn decay(
    env: ProcEnv,
    #[state] last_trig: &mut bool,
    #[state] value: &mut f32,
    #[state] time: &mut f32,

    #[input] trig: &bool,
    #[input] tau: &f32,

    #[output] out: &mut f32,
) -> ProcResult<()> {
    let tau = tau.max(0.0);

    if *trig && !*last_trig {
        *value = 1.0;
        *time = 0.0;
    } else if *value > 0.0 {
        *time += 1.0 / env.sample_rate;
        *value = (-tau.recip() * *time).exp();
    }

    *last_trig = *trig;
    *value = value.clamp(0.0, 1.0);
    *out = *value;

    Ok(())
}

impl Decay {
    pub fn new(tau: f32) -> Self {
        Self {
            last_trig: false,
            value: 0.0,
            time: 0.0,
            trig: false,
            tau,
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq, Debug)]
enum AdsrState {
    Attack,
    Decay,
    #[default]
    Sustain,
    Release,
}

#[processor]
pub fn adsr(
    env: ProcEnv,
    #[state] state: &mut AdsrState,
    #[state] last_gate: &mut bool,
    #[state] value: &mut f32,

    #[input] gate: &bool,
    #[input] attack: &f32,
    #[input] decay: &f32,
    #[input] sustain: &f32,
    #[input] release: &f32,

    #[output] out: &mut f32,
) -> ProcResult<()> {
    let attack = attack * env.sample_rate;
    let decay = decay * env.sample_rate;
    let release = release * env.sample_rate;

    if *gate && !*last_gate {
        *value = 0.0;
        *state = AdsrState::Attack;
    } else if !*gate && *last_gate {
        *state = AdsrState::Release;
    }

    let slope = match *state {
        AdsrState::Attack => {
            if attack > 0.0 {
                1.0 / attack
            } else {
                1.0
            }
        }
        AdsrState::Decay => {
            if decay > 0.0 {
                -(1.0 - *sustain) / decay
            } else {
                -1.0
            }
        }
        AdsrState::Sustain => 0.0,
        AdsrState::Release => {
            if release > 0.0 {
                -(1.0 - *sustain) / release
            } else {
                -1.0
            }
        }
    };

    *value += slope;

    if *state == AdsrState::Attack && *value >= 1.0 {
        *value = 1.0;
        *state = AdsrState::Decay;
    } else if *state == AdsrState::Decay && *value <= *sustain {
        *value = *sustain;
        *state = AdsrState::Sustain;
    } else if *state == AdsrState::Release && *value <= 0.0 {
        *value = 0.0;
        *state = AdsrState::Sustain;
    }

    *last_gate = *gate;
    *out = *value;

    Ok(())
}

impl Default for Adsr {
    fn default() -> Self {
        Self {
            state: AdsrState::Sustain,
            last_gate: false,
            value: 0.0,
            gate: false,
            attack: 0.0,
            decay: 0.0,
            sustain: 1.0,
            release: 0.0,
        }
    }
}

impl Adsr {
    pub fn new(attack: f32, decay: f32, sustain: f32, release: f32) -> Self {
        Self {
            attack,
            decay,
            sustain,
            release,
            ..Default::default()
        }
    }
}

pub trait IntoPattern<T: Signal> {
    fn into_pattern(self) -> List<T>;
}

impl<T: Signal> IntoPattern<T> for List<T> {
    #[inline]
    fn into_pattern(self) -> List<T> {
        self
    }
}

impl<T: Signal + Clone> IntoPattern<T> for &[T] {
    #[inline]
    fn into_pattern(self) -> List<T> {
        List::from_slice(self)
    }
}

impl<T: Signal + Clone, const N: usize> IntoPattern<T> for [T; N] {
    #[inline]
    fn into_pattern(self) -> List<T> {
        List::from_slice(&self)
    }
}

impl IntoPattern<bool> for &str {
    #[inline]
    fn into_pattern(self) -> List<bool> {
        let mut list = List::with_capacity(self.len());
        for slc in self.split_ascii_whitespace() {
            match slc {
                "." => list.push(false),
                _ => list.push(true),
            }
        }
        list
    }
}

#[processor(derive(Default))]
pub fn bool_pattern(
    #[state] index_state: &mut usize,
    #[input] trig: &bool,
    #[input] pattern: &Str,
    #[output] out: &mut bool,
    #[output] index: &mut f32,
) -> ProcResult<()> {
    let pattern = pattern.into_pattern();
    *index = *index_state as f32;

    if !pattern.is_empty() {
        if *trig {
            *out = pattern[*index_state];
            *index_state += 1;
            *index_state %= pattern.len();
        } else {
            *out = false;
        }
    }

    Ok(())
}

impl IntoPattern<f32> for &str {
    #[inline]
    fn into_pattern(self) -> List<f32> {
        let mut list = List::with_capacity(self.len());
        for slc in self.split_ascii_whitespace() {
            if let Ok(i) = slc.parse() {
                list.push(i)
            }
        }
        list
    }
}

impl<const N: usize> IntoPattern<f32> for [i32; N] {
    fn into_pattern(self) -> List<f32> {
        let mut list = List::default();
        for n in self {
            list.push(n as f32);
        }
        list
    }
}

#[processor]
pub fn pattern(
    #[state] index_state: &mut isize,
    #[input] trig: &bool,
    #[input] pattern: &Str,
    #[output] out: &mut f32,
    #[output] index: &mut f32,
) -> ProcResult<()> {
    let pattern = pattern.into_pattern();
    *index = *index_state as f32;

    if !pattern.is_empty() {
        if *index_state < 0 {
            *out = pattern[(pattern.len() as isize + *index_state) as usize];
        } else {
            *out = pattern[*index_state as usize];
        }
        if *trig {
            *index_state += 1;
            *index_state %= pattern.len() as isize;
        }
    }

    Ok(())
}

impl Default for Pattern {
    fn default() -> Self {
        Self {
            index_state: -1,
            trig: false,
            pattern: Str::new(),
        }
    }
}

#[inline]
const fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[inline]
const fn catmull_rom(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
    let a = (-0.5 * p0) + (1.5 * p1) - (1.5 * p2) + (0.5 * p3);
    let b = p0 - (2.5 * p1) + (2.0 * p2) - (0.5 * p3);
    let c = (-0.5 * p0) + (0.5 * p2);
    let d = p1;
    a * t * t * t + b * t * t + c * t + d
}

#[processor]
pub fn delay(
    env: ProcEnv,
    #[state] ringbuf: &mut Vec<f32>,
    #[state] write_index: &mut usize,
    #[input] input: &f32,
    #[input] delay: &f32,
    #[input] feedback: &f32,
    #[output] out: &mut f32,
) -> ProcResult<()> {
    let delay = delay.max(0.0);
    let delay_samples = delay * env.sample_rate;
    if delay_samples >= ringbuf.len() as f32 {
        ringbuf.resize((delay_samples + 3.0) as usize, 0.0); // we hate doing this here, but we have to
    }
    let read_index = *write_index as f32 - delay_samples;
    let read_index = if read_index < 0.0 {
        ringbuf.len() as f32 + read_index
    } else {
        read_index
    };

    let index1 = read_index.floor() as usize % ringbuf.len();
    let frac = read_index.fract();

    let index0 = (index1 + ringbuf.len() - 1) % ringbuf.len();
    let index2 = (index1 + 1) % ringbuf.len();
    let index3 = (index1 + 2) % ringbuf.len();

    let s0 = ringbuf[index0];
    let s1 = ringbuf[index1];
    let s2 = ringbuf[index2];
    let s3 = ringbuf[index3];

    *out = catmull_rom(s0, s1, s2, s3, frac);

    let feedback = feedback.clamp(-1.0, 1.0);
    ringbuf[*write_index] = *input + feedback * *out;
    *write_index = (*write_index + 1) % ringbuf.len();

    Ok(())
}

impl Default for Delay {
    fn default() -> Self {
        Self {
            ringbuf: vec![0.0; 1],
            write_index: 0,
            input: 0.0,
            feedback: 0.0,
            delay: 0.0,
        }
    }
}

impl Delay {
    pub fn new(delay: f32) -> Self {
        Self {
            ringbuf: vec![0.0; (delay * 48000.0) as usize + 1],
            write_index: 0,
            input: 0.0,
            feedback: 0.0,
            delay,
        }
    }
}

#[processor]
pub fn allpass(
    env: ProcEnv,
    #[state] ringbuf: &mut Vec<f32>,
    #[state] write_index: &mut usize,
    #[input] input: &f32,
    #[input] delay: &f32,
    #[input] gain: &f32,
    #[output] out: &mut f32,
) -> ProcResult<()> {
    let delay = delay.max(0.0);
    let delay_samples = delay * env.sample_rate;
    if delay_samples >= ringbuf.len() as f32 {
        ringbuf.resize((delay_samples + 1.0) as usize, 0.0); // we hate doing this here, but we have to
    }

    let delayed = ringbuf[*write_index];
    *out = -*input + delayed;
    ringbuf[*write_index] = *input + gain * delayed;

    *write_index = (*write_index + 1) % ringbuf.len();

    Ok(())
}

impl Allpass {
    pub fn new(delay: f32, gain: f32) -> Self {
        Self {
            ringbuf: vec![0.0; (delay * 48000.0) as usize + 1],
            write_index: 0,
            input: 0.0,
            delay,
            gain,
        }
    }
}

struct ReberbVoice {
    delay: Delay,
    allpass: Allpass,
    lfo: SineOscillator,
}

impl ReberbVoice {
    pub fn new(delay: f32, feedback: f32, diffusion_gain: f32, lfo_freq: f32) -> Self {
        Self {
            delay: Delay {
                feedback,
                ..Delay::new(delay)
            },
            allpass: Allpass::new(delay, diffusion_gain),
            lfo: SineOscillator {
                frequency: lfo_freq,
                ..Default::default()
            },
        }
    }

    pub fn process_sample(&mut self, env: ProcEnv, mut input: f32) -> ProcResult<f32> {
        let mut out = 0.0;
        let mut lfo_phase = 0.0;
        SineOscillator::process_sample(
            env,
            &mut self.lfo.t,
            &self.lfo.phase,
            &self.lfo.frequency,
            &false,
            &mut lfo_phase,
        )?;
        Delay::process_sample(
            env,
            &mut self.delay.ringbuf,
            &mut self.delay.write_index,
            &input,
            &(self.delay.delay + lfo_phase),
            &self.delay.feedback,
            &mut out,
        )?;
        input = out;
        Allpass::process_sample(
            env,
            &mut self.allpass.ringbuf,
            &mut self.allpass.write_index,
            &input,
            &self.allpass.delay,
            &self.allpass.gain,
            &mut out,
        )?;

        Ok(out)
    }
}

#[processor]
pub fn mono_reverb(
    env: ProcEnv,
    #[state] voices: &mut Vec<ReberbVoice>,
    #[input] input: &f32,
    #[output] out: &mut f32,
) -> ProcResult<()> {
    *out = 0.0;

    for voice in voices.iter_mut() {
        *out += voice.process_sample(env, *input)?;
    }

    *out /= voices.len() as f32;

    Ok(())
}

impl Default for MonoReverb {
    fn default() -> Self {
        Self {
            voices: vec![
                ReberbVoice::new(0.029, 0.7, 0.5, 0.1),
                ReberbVoice::new(0.037, 0.75, 0.5, 0.07),
                ReberbVoice::new(0.041, 0.72, 0.5, 0.11),
                ReberbVoice::new(0.053, 0.78, 0.5, 0.05),
            ],
            input: 0.0,
        }
    }
}

#[processor]
pub fn stereo_reverb(
    env: ProcEnv,
    #[state] voices_l: &mut Vec<ReberbVoice>,
    #[state] voices_r: &mut Vec<ReberbVoice>,
    #[input] input_l: &f32,
    #[input] input_r: &f32,
    #[input] crossfeed: &f32,
    #[output] out_l: &mut f32,
    #[output] out_r: &mut f32,
) -> ProcResult<()> {
    *out_l = 0.0;
    *out_r = 0.0;

    for (voice_l, voice_r) in voices_l.iter_mut().zip(voices_r.iter_mut()) {
        *out_l += voice_l.process_sample(env, *input_l)?;
        *out_r += voice_r.process_sample(env, *input_r)?;
    }

    *out_l /= voices_l.len() as f32;
    *out_r /= voices_r.len() as f32;

    let l = *out_l;
    let r = *out_r;

    *out_l = lerp(l, r, *crossfeed);
    *out_r = lerp(r, l, *crossfeed);

    Ok(())
}

impl Default for StereoReverb {
    fn default() -> Self {
        Self {
            voices_l: vec![
                ReberbVoice::new(0.029, 0.7, 0.5, 0.1),
                ReberbVoice::new(0.037, 0.75, 0.5, 0.07),
                ReberbVoice::new(0.041, 0.72, 0.5, 0.11),
                ReberbVoice::new(0.053, 0.78, 0.5, 0.05),
            ],
            voices_r: vec![
                ReberbVoice::new(0.028, 0.71, 0.5, 0.11),
                ReberbVoice::new(0.039, 0.74, 0.5, 0.08),
                ReberbVoice::new(0.040, 0.73, 0.5, 0.12),
                ReberbVoice::new(0.054, 0.77, 0.5, 0.04),
            ],
            input_l: 0.0,
            input_r: 0.0,
            crossfeed: 0.2,
        }
    }
}
