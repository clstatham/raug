use raug::prelude::*;

#[processor]
pub fn peak_limiter(
    #[state] gain: &mut f32,
    #[state] envelope: &mut f32,
    #[input] input: &f32,
    #[input] threshold: &f32,
    #[input] attack: &f32,
    #[input] release: &f32,
    #[output] out: &mut f32,
) -> ProcResult<()> {
    *envelope = (*input).abs().max(*envelope * *release);

    let target_gain = if *envelope > *threshold {
        *threshold / *envelope
    } else {
        1.0
    };

    *gain = *gain * *attack + target_gain * (1.0 - *attack);

    *out = *input * *gain;

    Ok(())
}

impl PeakLimiter {
    pub fn new(threshold: f32, attack: f32, release: f32) -> Self {
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
            input: 0.0,
            // -0.1 dBFS
            threshold: 0.988_553_1,
            attack: 0.9,
            release: 0.9995,
        }
    }
}
