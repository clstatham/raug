use std::f32::consts::PI;

use raug::prelude::*;
use raug_ext::prelude::*;
use raug_graph::node::NodeIndexExt;

/// A processor that generates Brownian noise.
#[processor(derive(Default))]
fn brownian(
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
    let phase = (PI * 2.0 * *phase as f32 * freq / env.sample_rate).sin();

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

impl Default for PlayingAround {
    fn default() -> Self {
        PlayingAround {
            state: 1.0,
            phase: 0,
            mod1: 0.0,
            mod2: 0.5,
            freq: 64.0,
        }
    }
}

fn main() {
    let mut graph = Graph::new();

    let mod1 = graph.add_node(Brownian::default());
    graph.connect_constant(200.0, mod1.input("speed"));

    let mod2 = graph.add_node(Brownian::default());
    graph.connect_constant(20.0, mod2.input("speed"));

    let osc = graph.add_node(PlayingAround::default());
    graph.connect(mod1, osc.input("mod1"));
    graph.connect(mod2, osc.input("mod2"));

    let hpf = graph.add_node(Biquad::highpass());
    // graph.connect_constant(20.0, hpf, "cutoff");
    // graph.connect_constant(1.0, hpf, "q");
    // graph.connect(osc, 0, hpf, 0);
    graph.connect_constant(20.0, hpf.input("cutoff"));
    graph.connect_constant(1.0, hpf.input("q"));
    graph.connect(osc, hpf.input(0));

    graph.connect_audio_output(hpf);
    graph.connect_audio_output(hpf);

    let running_graph = graph
        .play(
            CpalOut::spawn(&AudioBackend::Default, &AudioDevice::Default)
                .record_to_wav("playing_around.wav"),
        )
        .unwrap();

    let mut running_graph = Some(running_graph);

    ctrlc::set_handler(move || {
        let _ = running_graph.take().unwrap().stop();
        println!("Stopped");
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
