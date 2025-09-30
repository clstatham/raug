use std::f32::consts::PI;

use raug::prelude::*;

#[processor(derive(Default))]
pub fn phase_accumulator(
    #[state] t: &mut u32,
    #[input] increment: &f32,
    #[input] reset: &bool,
    #[output] out: &mut f32,
) -> ProcResult<()> {
    *out = *t as f32 * increment;

    if *reset {
        *t = 0;
    } else {
        *t += 1;
    }

    Ok(())
}

impl PhaseAccumulator {
    pub fn new(increment: f32) -> Self {
        PhaseAccumulator {
            increment,
            ..Default::default()
        }
    }
}

#[processor(derive(Default))]
pub fn sine_oscillator(
    env: ProcEnv,
    #[state] t: &mut f32,
    #[input] frequency: &f32,
    #[input] phase: &f32,
    #[input] reset: &bool,
    #[output] out: &mut f32,
) -> ProcResult<()> {
    if *reset {
        *t = 0.0;
        *out = 0.0;
    } else {
        *t += frequency / env.sample_rate * 2.0 * std::f32::consts::PI;
        *out = (*t + phase).sin();
    }
    *t %= 2.0 * std::f32::consts::PI;

    Ok(())
}

#[processor(derive(Default))]
pub fn noise_oscillator(#[output] out: &mut f32) -> ProcResult<()> {
    *out = rand::random::<f32>();
    Ok(())
}

#[processor]
pub fn bl_saw_oscillator(
    env: ProcEnv,
    #[state] p: &mut f32,
    #[state] dp: &mut f32,
    #[state] saw: &mut f32,
    #[input] frequency: &f32,
    #[output] out: &mut f32,
) -> ProcResult<()> {
    if *frequency <= 0.0 {
        *out = 0.0;
        return Ok(());
    }

    let pmax = 0.5 * env.sample_rate / frequency;
    let dc = -0.498 / pmax;

    *p += *dp;
    if *p < 0.0 {
        *p = -*p;
        *dp = -*dp;
    } else if *p > pmax {
        *p = 2.0 * pmax - *p;
        *dp = -*dp;
    }

    let mut x = PI * *p;
    if x < 0.00001 {
        x = 0.00001;
    }

    *saw = 0.995 * *saw + dc + x.sin() / x;
    *out = *saw;

    Ok(())
}

impl Default for BlSawOscillator {
    fn default() -> Self {
        BlSawOscillator {
            p: 0.0,
            dp: 1.0,
            saw: 0.0,
            frequency: 440.0,
        }
    }
}
