use std::f32::consts::PI;

use raug::prelude::*;
use raug_ext::prelude::*;

pub struct SmoothRandomLfoState {
    pub a: f32,
    pub a_n: f32,
    pub c_n: f32,
    pub last_a: f32,
    pub a_step: f32,
    pub x: f32,
    pub last_sign: f32,
    pub amp_scale: f32,
    pub new_amp_scale: f32,
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

/// A processor that generates Brownian noise.
#[processor(derive(Default))]
pub fn brownian(
    env: ProcEnv,
    #[state] last: &mut f32,
    #[state] last2: &mut f32,
    #[input] speed: &f32,
    #[output] out: &mut f32,
) -> ProcResult<()> {
    let delta: f32 = (rand::random::<f32>() * 2.0 - 1.0) * speed / env.sample_rate;

    *last += delta;

    // smooth the output a bit
    *last2 = 0.9 * *last2 + 0.1 * *last;

    *last2 = last2.tanh();

    *out = *last2;

    Ok(())
}

#[allow(clippy::collapsible_else_if)]
#[processor]
pub fn weird_tuba(
    env: ProcEnv,
    #[state] state: &mut f32,
    #[state] t: &mut f32,
    #[input] freq: &f32,
    #[input] mod1: &f32,
    #[input] mod2: &f32,
    #[output] out: &mut f32,
) -> ProcResult<()> {
    // phase accumulation for sine wave
    *t += freq / env.sample_rate;
    *t %= 1.0;
    let phase = (PI * 2.0 * *t).sin();

    // linear interpolation target
    let t = phase * *mod1;

    // integrate
    *state = *state + (phase * t - (1.0 - t) * *state) * *mod2;

    // non-linearity and clipping
    *state = state.tanh();

    // output
    *out = *state;

    Ok(())
}

impl Default for WeirdTuba {
    fn default() -> Self {
        WeirdTuba {
            state: 0.0,
            t: 0.0,
            mod1: 0.0,
            mod2: 0.5,
            freq: 32.0,
        }
    }
}

fn main() {
    let mut graph = Graph::new();

    let mut client = OscClient::bind("localhost:9000").unwrap();

    let osc = graph.node(WeirdTuba::default());
    let mod1 = graph.node(client.register_param("mod1", 0.0));
    let mod2 = graph.node(client.register_param("mod2", 0.0));
    let note = graph.node(client.register_param("note", 32.0));
    let vel = graph.node(client.register_param("vel", 0.0));

    let lfo1 = graph.node(Brownian::default());
    graph.connect(mod1, lfo1.input("speed"));
    let lfo2 = graph.node(Brownian::default());
    graph.connect(mod2, lfo2.input("speed"));

    let note = graph.smooth(note, 0.02);
    let freq = graph.node(PitchToFreq::default());
    graph.connect(note, freq.input("pitch"));

    graph.connect(lfo1, osc.input("mod1"));
    graph.connect(lfo2, osc.input("mod2"));
    graph.connect(freq, osc.input("freq"));

    let adsr = graph.node(Adsr::default());
    graph.connect(vel, adsr.input("gate"));
    graph.connect_constant(0.1, adsr.input("attack"));
    graph.connect_constant(0.1, adsr.input("decay"));
    graph.connect_constant(0.5, adsr.input("sustain"));
    graph.connect_constant(0.5, adsr.input("release"));

    let osc = graph.mul(osc, adsr);

    client.add_rule("/note_on", move |args, params| {
        let &[OscType::Int(note), OscType::Int(velocity)] = args else {
            return;
        };

        if let Some(p) = params.get("note") {
            p.set(note as f32);
        }

        if let Some(p) = params.get("vel") {
            let new_value = velocity as f32 / 127.0;
            p.set(new_value);
            println!("amp: {}", new_value);
        }
    });

    client.add_rule("/note_off", move |args, params| {
        let &[OscType::Int(_note), OscType::Int(_velocity)] = args else {
            return;
        };

        if let Some(p) = params.get("vel") {
            p.set(0.0);
            println!("amp: 0.0");
        }
    });

    client.add_rule("/control_change", move |args, params| {
        let &[OscType::Int(cc), OscType::Int(value)] = args else {
            return;
        };

        let normalized = value as f32 / 127.0;

        match cc {
            70 => {
                if let Some(p) = params.get("mod1") {
                    let new_value = normalized * 200.0;
                    p.set(new_value);
                    println!("mod1: {}", new_value);
                }
            }
            71 => {
                if let Some(p) = params.get("mod2") {
                    let new_value = normalized * 20.0;
                    p.set(new_value);
                    println!("mod2: {}", new_value);
                }
            }
            _ => {}
        }
    });

    let hpf = graph.node(Biquad::highpass());
    graph.connect_constant(20.0, hpf.input("cutoff"));
    graph.connect_constant(1.0, hpf.input("q"));
    graph.connect(osc, hpf.input(0));

    let mast = graph.node(hpf * 1.0);

    graph.connect_audio_output(mast);
    graph.connect_audio_output(mast);

    let stream = CpalOut::spawn(&AudioBackend::Default, &AudioDevice::Default, None)
        .record_to_wav("playing_around.wav");

    let kill_switch = KillSwitch::default();
    ctrlc::set_handler({
        let kill_switch = kill_switch.clone();
        move || {
            kill_switch.kill();
        }
    })
    .unwrap();

    client.spawn();

    graph.play(&stream, Some(kill_switch)).unwrap();
}
