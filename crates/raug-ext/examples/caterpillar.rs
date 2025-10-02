use raug::prelude::*;
use raug_ext::prelude::*;

use std::f32::consts::PI;

pub fn pick_randomly(graph: &mut Graph, trig: NodeIndex, options: &[f32]) -> NodeIndex {
    let node = graph.node(RandomChoice::<f32>::default());
    graph.connect(trig, node.input("trig"));
    let options = graph.constant(List::from_slice(options));
    graph.connect(options, node.input("options"));
    node
}

pub fn fm_sine_osc(graph: &mut Graph, freq: NodeIndex, modulator: NodeIndex) -> NodeIndex {
    let sr = graph.node(SampleRate::default());
    let phase = graph.node(PhaseAccumulator::default());
    let freq_over_sr = graph.div(freq, sr);
    graph.connect(freq_over_sr, phase.input("increment"));
    let phase_times_2pi = graph.mul(phase, 2.0 * PI);
    let mod_phase_times_2pi = graph.mul(modulator, 2.0 * PI);
    let modulated_phase = graph.add(phase_times_2pi, mod_phase_times_2pi);
    graph.sin(modulated_phase)
}

pub fn midi_to_freq(midi: f32) -> f32 {
    440.0 * 2.0_f32.powf((midi - 69.0) / 12.0)
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
    graph: &mut Graph,
    rates: &[f32],
    ratios: &[f32],
    freqs: &[f32],
    decays: &[f32],
    amps: &[f32],
) -> NodeIndex {
    // let mast = Metro::default().node(graph, rates[0], ());
    let mast = graph.node(Metro::default());
    graph.connect_constant(rates[0], mast.input("period"));

    // let rate = pick_randomly(graph, &mast, rates);
    let rate = pick_randomly(graph, mast, rates);

    // let trig = Metro::default().node(graph, rate, ());
    let trig = graph.node(Metro::default());
    graph.connect(rate, trig.input("period"));

    // let freq = pick_randomly(graph, &trig, freqs);
    let freq = pick_randomly(graph, trig, freqs);

    // let amp_decay = pick_randomly(graph, &trig, decays);
    let amp_decay = pick_randomly(graph, trig, decays);

    // let ratio = pick_randomly(graph, &trig, ratios);
    let ratio = pick_randomly(graph, trig, ratios);

    // let amp = pick_randomly(graph, &trig, amps);
    let amp = pick_randomly(graph, trig, amps);

    // create the amplitude envelope
    // let amp_env = Decay::new(1.0f32).node(graph, &trig, amp_decay);
    let amp_env = graph.node(Decay::new(1.0f32));
    graph.connect(trig, amp_env.input("trig"));
    graph.connect(amp_decay, amp_env.input("tau"));

    // create the modulator
    // let modulator = BlSawOscillator::default().node(graph, &freq * ratio);
    let modulator = graph.node(BlSawOscillator::default());
    let mod_freq = graph.mul(freq, ratio);
    graph.connect(mod_freq, modulator.input("frequency"));

    // create the carrier
    // let carrier = fm_sine_osc(graph, &freq, &(modulator * 0.1f32));
    let modulator = graph.mul(modulator, 0.1);
    let carrier = fm_sine_osc(graph, freq, modulator);

    // carrier * amp_env * amp
    let carrier_amp = graph.mul(carrier, amp_env);

    graph.mul(carrier_amp, amp)
}

pub fn caterpillar(num_tones: usize) -> Graph {
    let ratios = &[0.25, 0.5, 1.0, 2.0];
    let decays = &[0.02, 0.1, 0.2, 0.5];
    let amps = &[0.125, 0.25, 0.5, 0.8];
    let rates = &[1. / 8., 1. / 4., 1. / 2., 1., 2.];

    let freqs = scale_freqs(24.0);

    let mut graph = Graph::new();

    let mut tones = vec![];
    for _ in 0..num_tones {
        let tone = random_tones(&mut graph, rates, ratios, &freqs, decays, amps);
        tones.push(tone);
    }

    let mut mix = tones[0];
    for tone in tones.iter().skip(1) {
        // mix = mix.clone() + tone.clone();
        mix = graph.add(mix, *tone);
    }

    // let mix = mix * 0.1f32;
    mix = graph.mul(mix, 0.1f32);

    let verb = graph.node(StereoReverb::default());
    graph.connect(mix, verb.input("input_l"));
    graph.connect(mix, verb.input("input_r"));
    let mix_l = graph.add(verb.output(0), mix);
    let mix_l = graph.mul(mix_l, 0.1f32);
    let mix_r = graph.add(verb.output(1), mix);
    let mix_r = graph.mul(mix_r, 0.1f32);

    let master_l = graph.node(PeakLimiter::default());
    graph.connect(mix_l, master_l.input("input"));
    let master_r = graph.node(PeakLimiter::default());
    graph.connect(mix_r, master_r.input("input"));

    graph.connect_audio_output(master_l);
    graph.connect_audio_output(master_r);

    graph
}

fn main() {
    let graph = caterpillar(250);

    graph
        .write_dot(&mut std::fs::File::create("caterpillar.dot").unwrap())
        .unwrap();

    println!("nodes: {}", graph.node_count());
    println!("edges: {}", graph.edge_count());

    graph
        // .play(CpalOut::spawn(
        //     &AudioBackend::Default,
        //     &AudioDevice::Default,
        // ))
        .play(NullOut::new(48_000.0, 512, 2))
        .unwrap()
        .run_for(Duration::from_secs(100))
        .unwrap();
}
