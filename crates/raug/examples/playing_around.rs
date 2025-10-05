use std::f32::consts::PI;

use raug::prelude::*;
// use raug_ext::prelude::*;

struct SmoothRandomLfoState {
    a: f32,
    a_n: f32,
    c_n: f32,
    last_a: f32,
    a_step: f32,
    x: f32,
    last_sign: f32,
    amp_scale: f32,
    new_amp_scale: f32,
}

// https://www.musicdsp.org/en/latest/Synthesis/269-smooth-random-lfo-generator.html
#[processor]
fn smooth_random_lfo(
    env: ProcEnv,
    #[state] state: &mut SmoothRandomLfoState,
    #[input] rate: &f32,
    #[output] out: &mut f32,
) -> ProcResult<()> {
    if *rate == 0.0 || env.sample_rate == 0.0 {
        // invalid state
        *out = 0.0;
        return Ok(());
    }

    let SmoothRandomLfoState {
        a,
        a_n,
        c_n,
        last_a,
        a_step,
        x,
        last_sign,
        amp_scale,
        new_amp_scale,
    } = state;

    let step_freq_scale = env.sample_rate / rate;
    let min_cn = step_freq_scale * 0.1;
    let amp_scale_ramp = (1000.0 / env.sample_rate).exp_m1();

    if *a_n == 0.0 || *a_n >= *c_n {
        *c_n = (step_freq_scale * rand::random::<f32>()).floor();
        *c_n = f32::max(*c_n, min_cn);
        let new_a = 0.1 + 0.9 * rand::random::<f32>();
        *a_step = (new_a - *last_a) / *c_n;
        *a = *last_a;
        *last_a = new_a;
        *a_n = 0.0;
    }

    *a_n += 1.0;
    *out = x.sin() * *amp_scale;
    *amp_scale += (*new_amp_scale - *amp_scale) * amp_scale_ramp;
    let sin_inc = 2.0 * PI * rate * *a / env.sample_rate;
    *a += *a_step;
    *x += sin_inc;
    *x %= 2.0 * PI;

    if out.signum() != *last_sign {
        *last_sign = out.signum();
        *new_amp_scale = 0.25 + 0.75 * rand::random::<f32>();
    }

    Ok(())
}

impl Default for SmoothRandomLfo {
    fn default() -> Self {
        SmoothRandomLfo {
            state: SmoothRandomLfoState {
                a: 0.0,
                a_n: 0.0,
                c_n: 0.0,
                last_a: 0.0,
                a_step: 0.0,
                x: 0.0,
                last_sign: 1.0,
                amp_scale: 0.0,
                new_amp_scale: 1.0,
            },
            rate: 1.0,
        }
    }
}

#[allow(clippy::collapsible_else_if)]
#[processor]
pub fn playing_around(
    env: ProcEnv,
    #[state] state: &mut f32,
    #[state] phase: &mut u32,
    #[input] freq: &f32,
    #[input] mod1: &f32,
    #[input] mod2: &f32,
    #[output] out: &mut f32,
) -> ProcResult<()> {
    // phase accumulation for sine wave
    *phase += 1;
    *phase %= env.sample_rate as u32;

    // sine wave
    let sine = (PI * 2.0 * *phase as f32 * freq / env.sample_rate).sin();

    // sine wave (octave)
    let sine2 = (PI * 2.0 * *phase as f32 * freq * 2.0 / env.sample_rate).sin();

    // integrate
    {
        let t = *mod1;
        let input = sine * t + sine2 * (1.0 - t);
        *state += input;
    }

    // non-linearity and clipping
    // *state = state.tanh();
    *state = state.tanh();

    // output
    *out = *state;

    Ok(())
}

impl Default for PlayingAround {
    fn default() -> Self {
        PlayingAround {
            state: 0.0,
            phase: 0,
            mod1: 0.0,
            mod2: 0.5,
            freq: 32.0,
        }
    }
}

fn main() {
    let mut graph = Graph::new();

    let mod1 = graph.node(SmoothRandomLfo::default());
    graph.connect_constant(0.5, mod1.input("rate"));

    let mod2 = graph.node(SmoothRandomLfo::default());
    graph.connect_constant(0.5, mod2.input("rate"));

    let osc = graph.node(PlayingAround::default());
    graph.connect(mod1, osc.input("mod1"));
    graph.connect(mod2, osc.input("mod2"));

    // let hpf = graph.node(Biquad::highpass());
    // graph.connect_constant(20.0, hpf.input("cutoff"));
    // graph.connect_constant(1.0, hpf.input("q"));
    // graph.connect(osc, hpf.input(0));

    let hpf = osc;

    graph.connect_audio_output(hpf);
    graph.connect_audio_output(hpf);

    graph
        .play(
            &mut CpalOut::spawn(
                &AudioBackend::Default,
                &AudioDevice::Default,
                Some(Duration::from_secs(10)),
            )
            .record_to_wav("playing_around.wav"),
            // WavFileOut::new("playing_around.wav", 48_000.0, 512, 2, None),
        )
        .unwrap();
}
