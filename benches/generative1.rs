use std::f32::consts::PI;

use raug::prelude::*;
use raug_ext::prelude::*;

pub fn pick_randomly(graph: &Graph, trig: &Node, options: &'static [f32]) -> Node {
    let node = graph.add(RandomChoice::<f32>::default());
    node.input("trig").connect(trig.output(0));
    node.input("options")
        .connect(graph.constant(List::from_slice(options)));
    node
}

pub fn fm_sine_osc(graph: &Graph, freq: &Node, mod_freq: &Node) -> Node {
    let sr = graph.add(SampleRate::default());
    let phase = graph.add(PhaseAccumulator::default());
    let increment = freq / sr;
    phase.input(0).connect(increment.output(0));
    (phase * 2.0f32 * PI + mod_freq * 2.0f32 * PI).sin()
}

pub fn random_tones(
    graph: &Graph,
    rates: &'static [f32],
    ratios: &'static [f32],
    freqs: &'static [f32],
    decays: &'static [f32],
    amps: &'static [f32],
) -> Node {
    let mast = graph.add(Metro::default());
    mast.input(0).connect(rates[0]);

    let rate = pick_randomly(graph, &mast, rates).unwrap_or(0.0f32);

    let trig = graph.add(Metro::default());
    trig.input(0).connect(rate.output(0));

    let freq = pick_randomly(graph, &trig, freqs).unwrap_or(440.0f32);

    let amp_decay = pick_randomly(graph, &trig, decays).unwrap_or(0.0f32);

    let ratio = pick_randomly(graph, &trig, ratios).unwrap_or(0.0f32);

    let amp = pick_randomly(graph, &trig, amps).unwrap_or(0.0f32);

    // create the amplitude envelope
    let amp_env = graph.add(DecayEnv::new(1.0f32));
    amp_env.input("tau").connect(amp_decay.output(0));
    amp_env.input("trig").connect(trig.output(0));

    // create the modulator
    let modulator = graph.add(BlSawOscillator::default());
    modulator.input(0).connect((&freq * ratio).output(0));

    // create the carrier
    let carrier = fm_sine_osc(graph, &freq, &(modulator * 0.5f32));

    carrier * amp_env * amp
}

pub fn generative1(num_tones: usize) -> Graph {
    let ratios = &[0.25, 0.5, 1.0, 2.0];
    let decays = &[0.02, 0.1, 0.2, 0.5];
    let amps = &[0.125, 0.25, 0.5, 0.8];
    let rates = &[1. / 8., 1. / 4., 1. / 2., 1., 2.];

    let freqs = &[
        261.62555, 293.66476, 311.12698, 349.22824, 391.99542, 415.3047, 466.1638, 523.2511,
        587.3295, 622.25397, 698.4565, 783.99084, 830.6094, 932.3276, 130.81277, 146.83238,
        155.56349, 174.61412, 195.99773, 207.65234, 233.08186,
    ];

    let graph = Graph::new();

    let out1 = graph.add_audio_output();
    let out2 = graph.add_audio_output();

    let mut tones = vec![];
    for _ in 0..num_tones {
        let tone = random_tones(&graph, rates, ratios, freqs, decays, amps);
        tones.push(tone);
    }

    let mut mix = tones[0].clone();
    for tone in tones.iter().skip(1) {
        mix = mix.clone() + tone.clone();
    }

    let mix = mix * 0.5f32;

    // let master = mix;

    let master = graph.add(PeakLimiter::default());
    master.input(0).connect(mix.output(0));

    master.output(0).connect(&out1.input(0));
    master.output(0).connect(&out2.input(0));

    graph
}
