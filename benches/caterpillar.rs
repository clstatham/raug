use std::f32::consts::PI;

use raug::prelude::*;
use raug_ext::prelude::*;

pub fn pick_randomly(graph: &Graph, trig: &Node, options: &[f32]) -> Node {
    RandomChoice::<f32>::default().node(graph, trig, options)
}

pub fn fm_sine_osc(graph: &Graph, freq: &Node, mod_freq: &Node) -> Node {
    let sr = SampleRate::default().node(graph);
    let phase = PhaseAccumulator::default().node(graph, freq / sr, ());
    (phase * 2.0f32 * PI + mod_freq * 2.0f32 * PI)
        .output(0)
        .sin()
}

pub fn midi_to_freq(midi: f32) -> f32 {
    440.0 * f32::powf(2.0, (midi - 69.0) / 12.0)
}

pub fn scale_freqs(detune: f32) -> Vec<f32> {
    // minor scale
    let scale = [0, 2, 3, 5, 7, 8, 10];
    let base = 48; // C3
    let mut freqs = vec![];
    for note in &scale {
        freqs.push(midi_to_freq(base as f32 + *note as f32 + detune));
    }
    let base = 60;
    for note in &scale {
        freqs.push(midi_to_freq(base as f32 + *note as f32 + detune));
    }
    let base = 36;
    for note in &scale {
        freqs.push(midi_to_freq(base as f32 + *note as f32 + detune));
    }
    freqs
}

pub fn random_tones(
    graph: &Graph,
    rates: &[f32],
    ratios: &[f32],
    freqs: &[f32],
    decays: &[f32],
    amps: &[f32],
) -> Node {
    let mast = Metro::default().node(graph, rates[0], ());

    let rate = pick_randomly(graph, &mast, rates);

    let trig = Metro::default().node(graph, rate, ());

    let freq = pick_randomly(graph, &trig, freqs);

    let amp_decay = pick_randomly(graph, &trig, decays);

    let ratio = pick_randomly(graph, &trig, ratios);

    let amp = pick_randomly(graph, &trig, amps);

    // create the amplitude envelope
    let amp_env = Decay::new(1.0f32).node(graph, &trig, amp_decay);

    // create the modulator
    let modulator = BlSawOscillator::default().node(graph, &freq * ratio);

    // create the carrier
    let carrier = fm_sine_osc(graph, &freq, &(modulator * 0.1f32));

    carrier * amp_env * amp
}

pub fn caterpillar(num_tones: usize) -> Graph {
    let ratios = &[0.25, 0.5, 1.0, 2.0];
    let decays = &[0.02, 0.1, 0.2, 0.5];
    let amps = &[0.125, 0.25, 0.5, 0.8];
    let rates = &[1. / 8., 1. / 4., 1. / 2., 1., 2.];

    let freqs = scale_freqs(0.0);

    let graph = Graph::new(0, 2);

    let mut tones = vec![];
    for _ in 0..num_tones {
        let tone = random_tones(&graph, rates, ratios, &freqs, decays, amps);
        tones.push(tone);
    }

    let mut mix = tones[0].clone();
    for tone in tones.iter().skip(1) {
        mix = mix.clone() + tone.clone();
    }

    let mix = mix * 0.1f32;

    let master = PeakLimiter::default().node(&graph, mix, (), (), ());

    graph.dac((&master, &master));

    graph
}
