use raug::prelude::*;
use std::f32::consts::PI;

#[processor]
pub fn lowpass1(
    env: ProcEnv,
    #[state] prev_out: &mut f32,
    #[input] input: &f32,
    #[input] cutoff: &f32,
    #[output] out: &mut f32,
) -> ProcResult<()> {
    let coeff = 1.0 - (-2.0 * PI * cutoff / env.sample_rate).exp();
    *prev_out = coeff * input + (1.0 - coeff) * *prev_out;
    *out = *prev_out;
    Ok(())
}

#[processor]
pub fn highpass1(
    env: ProcEnv,
    #[state] prev_out: &mut f32,
    #[state] prev_in: &mut f32,
    #[input] input: &f32,
    #[input] cutoff: &f32,
    #[output] out: &mut f32,
) -> ProcResult<()> {
    let coeff = (-2.0 * PI * cutoff / env.sample_rate).exp();
    *prev_out = coeff * (*prev_out + input - *prev_in);
    *prev_in = *input;
    *out = *prev_out;
    Ok(())
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum BiquadMode {
    #[default]
    Lowpass,
    Highpass,
    Bandpass,
    Notch,
    Allpass,
    LowShelf,
    HighShelf,
    Peaking,
}

impl Signal for BiquadMode {}

#[derive(Debug, Clone, Copy)]
pub struct BiquadCommon {
    pub cos_omega: f32,
    pub alpha: f32,
}

impl BiquadCommon {
    #[inline]
    pub fn new(cutoff: f32, q: f32, sample_rate: f32) -> Self {
        let omega = 2.0 * PI * cutoff / sample_rate;
        let alpha = omega / (2.0 * q);
        let cos_omega = omega.cos();

        Self { cos_omega, alpha }
    }
}

#[derive(Default, Clone, Copy)]
#[allow(unused)]
pub struct BiquadState {
    pub mode: BiquadMode,
    pub a1: f32,
    pub a2: f32,
    pub b0: f32,
    pub b1: f32,
    pub b2: f32,
    pub prev_out: [f32; 2],
    pub prev_in: [f32; 2],
}

impl BiquadState {
    #[inline]
    fn normalize(&mut self, a0: f32) {
        assert_ne!(a0, 0.0, "a0 cannot be zero");
        self.a1 /= a0;
        self.a2 /= a0;
        self.b0 /= a0;
        self.b1 /= a0;
        self.b2 /= a0;
    }

    #[inline]
    pub fn update(&mut self, cutoff: f32, q: f32, amp: f32, sample_rate: f32) {
        let cutoff = cutoff.clamp(0.0, sample_rate / 2.0); // must be > 0 and < nyquist
        let q = q.max(0.001); // must be > 0
        let amp = amp.max(0.001); // must be > 0
        match self.mode {
            BiquadMode::Lowpass => self.lowpass(cutoff, q, sample_rate),
            BiquadMode::Highpass => self.highpass(cutoff, q, sample_rate),
            BiquadMode::Bandpass => self.bandpass(cutoff, q, sample_rate),
            BiquadMode::Notch => self.notch(cutoff, q, sample_rate),
            BiquadMode::Allpass => self.allpass(cutoff, q, sample_rate),
            BiquadMode::LowShelf => self.lowshelf(cutoff, q, amp, sample_rate),
            BiquadMode::HighShelf => self.highshelf(cutoff, q, amp, sample_rate),
            BiquadMode::Peaking => self.peaking(cutoff, q, amp, sample_rate),
        }
    }

    pub fn lowpass(&mut self, cutoff: f32, q: f32, sample_rate: f32) {
        let BiquadCommon { cos_omega, alpha } = BiquadCommon::new(cutoff, q, sample_rate);

        let a0 = 1.0 + alpha;
        self.b0 = (1.0 - cos_omega) / 2.0;
        self.b1 = 1.0 - cos_omega;
        self.b2 = (1.0 - cos_omega) / 2.0;
        self.a1 = -2.0 * cos_omega;
        self.a2 = 1.0 - alpha;

        self.normalize(a0);
    }

    pub fn highpass(&mut self, cutoff: f32, q: f32, sample_rate: f32) {
        let BiquadCommon { cos_omega, alpha } = BiquadCommon::new(cutoff, q, sample_rate);

        let a0 = 1.0 + alpha;
        self.b0 = (1.0 + cos_omega) / 2.0;
        self.b1 = -(1.0 + cos_omega);
        self.b2 = (1.0 + cos_omega) / 2.0;
        self.a1 = -2.0 * cos_omega;
        self.a2 = 1.0 - alpha;

        self.normalize(a0);
    }

    pub fn bandpass(&mut self, cutoff: f32, q: f32, sample_rate: f32) {
        let BiquadCommon { cos_omega, alpha } = BiquadCommon::new(cutoff, q, sample_rate);

        let a0 = 1.0 + alpha;
        self.b0 = alpha;
        self.b1 = 0.0;
        self.b2 = -alpha;
        self.a1 = -2.0 * cos_omega;
        self.a2 = 1.0 - alpha;

        self.normalize(a0);
    }

    pub fn notch(&mut self, cutoff: f32, q: f32, sample_rate: f32) {
        let BiquadCommon { cos_omega, alpha } = BiquadCommon::new(cutoff, q, sample_rate);

        let a0 = 1.0 + alpha;
        self.b0 = 1.0;
        self.b1 = -2.0 * cos_omega;
        self.b2 = 1.0;
        self.a1 = -2.0 * cos_omega;
        self.a2 = 1.0 - alpha;

        self.normalize(a0);
    }

    pub fn allpass(&mut self, cutoff: f32, q: f32, sample_rate: f32) {
        let BiquadCommon { cos_omega, alpha } = BiquadCommon::new(cutoff, q, sample_rate);

        let a0 = 1.0 + alpha;
        self.b0 = 1.0 - alpha;
        self.b1 = -2.0 * cos_omega;
        self.b2 = 1.0 + alpha;
        self.a1 = -2.0 * cos_omega;
        self.a2 = 1.0 - alpha;

        self.normalize(a0);
    }

    pub fn lowshelf(&mut self, cutoff: f32, q: f32, amp: f32, sample_rate: f32) {
        let BiquadCommon { cos_omega, alpha } = BiquadCommon::new(cutoff, q, sample_rate);
        let sqrt_amp = amp.sqrt();

        let a0 = (amp + 1.0) - (amp - 1.0) * cos_omega + 2.0 * sqrt_amp * alpha;
        self.b0 = amp * ((amp + 1.0) - (amp - 1.0) * cos_omega + 2.0 * sqrt_amp * alpha);
        self.b1 = 2.0 * amp * ((amp - 1.0) - (amp + 1.0) * cos_omega);
        self.b2 = amp * ((amp + 1.0) - (amp - 1.0) * cos_omega - 2.0 * sqrt_amp * alpha);
        self.a1 = -2.0 * ((amp - 1.0) + (amp + 1.0) * cos_omega);
        self.a2 = (amp + 1.0) - (amp - 1.0) * cos_omega - 2.0 * sqrt_amp * alpha;

        self.normalize(a0);
    }

    pub fn highshelf(&mut self, cutoff: f32, q: f32, amp: f32, sample_rate: f32) {
        let BiquadCommon { cos_omega, alpha } = BiquadCommon::new(cutoff, q, sample_rate);
        let sqrt_amp = amp.sqrt();

        let a0 = (amp + 1.0) + (amp - 1.0) * cos_omega + 2.0 * sqrt_amp * alpha;
        self.b0 = amp * ((amp + 1.0) + (amp - 1.0) * cos_omega + 2.0 * sqrt_amp * alpha);
        self.b1 = -2.0 * amp * ((amp - 1.0) + (amp + 1.0) * cos_omega);
        self.b2 = amp * ((amp + 1.0) + (amp - 1.0) * cos_omega - 2.0 * sqrt_amp * alpha);
        self.a1 = -2.0 * ((amp - 1.0) - (amp + 1.0) * cos_omega);
        self.a2 = (amp + 1.0) + (amp - 1.0) * cos_omega - 2.0 * sqrt_amp * alpha;

        self.normalize(a0);
    }

    pub fn peaking(&mut self, cutoff: f32, q: f32, amp: f32, sample_rate: f32) {
        let BiquadCommon { cos_omega, alpha } = BiquadCommon::new(cutoff, q, sample_rate);

        let a0 = 1.0 + alpha / amp;
        self.b0 = 1.0 + alpha * amp;
        self.b1 = -2.0 * cos_omega;
        self.b2 = 1.0 - alpha * amp;
        self.a1 = -2.0 * cos_omega;
        self.a2 = 1.0 - alpha / amp;

        self.normalize(a0);
    }
}

#[processor(derive(Default))]
pub fn biquad(
    env: ProcEnv,
    #[state] state: &mut BiquadState,
    #[input] input: &f32,
    #[input] cutoff: &f32,
    #[input] q: &f32,
    #[input] amp: &f32,
    #[output] out: &mut f32,
) -> ProcResult<()> {
    state.update(*cutoff, *q, *amp, env.sample_rate);

    *out = state.b0 * input + state.b1 * state.prev_in[0] + state.b2 * state.prev_in[1]
        - state.a1 * state.prev_out[0]
        - state.a2 * state.prev_out[1];

    state.prev_in[1] = state.prev_in[0];
    state.prev_in[0] = *input;
    state.prev_out[1] = state.prev_out[0];
    state.prev_out[0] = *out;

    Ok(())
}

impl Biquad {
    pub fn lowpass() -> Self {
        Self {
            state: BiquadState {
                mode: BiquadMode::Lowpass,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn highpass() -> Self {
        Self {
            state: BiquadState {
                mode: BiquadMode::Highpass,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn bandpass() -> Self {
        Self {
            state: BiquadState {
                mode: BiquadMode::Bandpass,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn notch() -> Self {
        Self {
            state: BiquadState {
                mode: BiquadMode::Notch,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn allpass() -> Self {
        Self {
            state: BiquadState {
                mode: BiquadMode::Allpass,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn lowshelf() -> Self {
        Self {
            state: BiquadState {
                mode: BiquadMode::LowShelf,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn highshelf() -> Self {
        Self {
            state: BiquadState {
                mode: BiquadMode::HighShelf,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn peaking() -> Self {
        Self {
            state: BiquadState {
                mode: BiquadMode::Peaking,
                ..Default::default()
            },
            ..Default::default()
        }
    }
}
