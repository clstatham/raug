use std::f32::consts::PI;

use raug::prelude::*;

pub fn random(graph: &Graph, trig: &Node) -> Node {
    let noise = graph.add(NoiseOscillator::new());
    let snh = graph.add(SampleAndHold::default());
    trig.output(0).connect(&snh.input("trig"));
    noise.output(0).connect(&snh.input("input"));
    snh
}

pub fn pick_randomly(graph: &Graph, trig: &Node, options: &[Node]) -> Node {
    let index = random(graph, trig);
    let index = index * (options.len() + 1) as f32;
    let index = index % options.len() as f32;
    let index = index.cast(SignalType::Int);

    let select = graph.add(Select::new(SignalType::Bool, options.len()));
    select.input("in").connect(graph.constant(true).output(0));
    select.input("index").connect(index.output(0));

    let merge = graph.add(Merge::new(SignalType::Float, options.len()));

    merge.input("index").connect(index.output(0));

    let msgs = options
        .iter()
        .map(|_| graph.add(Message::new(0.0)))
        .collect::<Vec<_>>();

    for (i, (option, msg)) in options.iter().zip(msgs.iter()).enumerate() {
        msg.input(0).connect(select.output(i as u32));
        msg.input(1).connect(option.output(0));
        merge.input(i as u32 + 1).connect(msg.output(0));
    }

    merge
}

pub fn fm_sine_osc(graph: &Graph, freq: &Node, mod_freq: &Node) -> Node {
    let sr = graph.add(SampleRate);
    let phase = graph.add(PhaseAccumulator::default());
    let increment = freq / sr;
    phase.input(0).connect(increment.output(0));
    (phase * 2.0 * PI + mod_freq * 2.0 * PI).sin()
}

pub fn midi_to_freq(midi: f32) -> f32 {
    440.0 * f32::powf(2.0, (midi - 69.0) / 12.0)
}

pub fn scale_freqs(detune: f32) -> Vec<f32> {
    // minor scale
    let scale = [0, 2, 3, 5, 7, 8, 10];
    let base = 60; // C4
    let mut freqs = vec![];
    for note in &scale {
        freqs.push(midi_to_freq(base as f32 + *note as f32 + detune));
    }
    let base = 72;
    for note in &scale {
        freqs.push(midi_to_freq(base as f32 + *note as f32 + detune));
    }
    let base = 48;
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
    let mast = graph.add(Metro::default());
    mast.input(0).connect(rates[0]);

    // select a random rate
    let rates = rates.iter().map(|&r| graph.constant(r)).collect::<Vec<_>>();
    let rate = pick_randomly(graph, &mast, &rates);

    let trig = graph.add(Metro::default());
    trig.input(0).connect(rate.output(0));

    // select a random frequency
    let freqs = freqs.iter().map(|&f| graph.constant(f)).collect::<Vec<_>>();
    let freq = pick_randomly(graph, &trig, &freqs);

    // select a random decay
    let amp_decays = decays
        .iter()
        .map(|&d| graph.constant(d))
        .collect::<Vec<_>>();
    let amp_decay = pick_randomly(graph, &trig, &amp_decays);

    // select a random mod ratio
    let ratios = ratios
        .iter()
        .map(|&r| graph.constant(r))
        .collect::<Vec<_>>();
    let ratio = pick_randomly(graph, &trig, &ratios);

    // select a random amplitude
    let amps = amps.iter().map(|&a| graph.constant(a)).collect::<Vec<_>>();
    let amp = pick_randomly(graph, &trig, &amps);

    // create the amplitude envelope
    // let amp_env = decay_env(graph, &trig, &amp_decay);
    let amp_env = graph.add(DecayEnv::new(1.0));
    amp_env.input("tau").connect(amp_decay.output(0));
    amp_env.input("trig").connect(trig.output(0));

    // create the modulator
    let modulator = graph.add(BlSawOscillator::default());
    modulator.input(0).connect((&freq * ratio).output(0));

    // create the carrier
    let carrier = fm_sine_osc(graph, &freq, &(modulator * 0.1));

    carrier * amp_env * amp
}

pub fn generative1(num_tones: usize) -> Graph {
    let ratios = [0.25, 0.5, 1.0, 2.0];
    let decays = [0.02, 0.1, 0.2, 0.5];
    let amps = [0.125, 0.25, 0.5, 0.8];
    let rates = [1. / 8., 1. / 4., 1. / 2., 1., 2.];

    let graph = Graph::new();

    let out1 = graph.add_audio_output();
    let out2 = graph.add_audio_output();

    // let amp = graph.add_param(Param::new::<f32>("amp", Some(0.5)));

    let mut tones = vec![];
    for _ in 0..num_tones {
        let freqs = scale_freqs(0.0);
        let tone = random_tones(&graph, &rates, &ratios, &freqs, &decays, &amps);
        tones.push(tone);
    }

    let mut mix = tones[0].clone();
    for tone in tones.iter().skip(1) {
        mix = mix.clone() + tone.clone();
    }

    let mix = mix * 0.5;

    // let master = mix;

    let master = graph.add(PeakLimiter::default());
    master.input(0).connect(mix.output(0));

    master.output(0).connect(&out1.input(0));
    master.output(0).connect(&out2.input(0));

    graph
}
