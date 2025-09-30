use raug::prelude::*;
use raug_ext::prelude::*;

#[allow(clippy::too_many_arguments)]
pub fn synth(
    graph: &Graph,
    pitch: impl IntoOutputExt,
    detune: impl IntoOutputExt,
    gate: impl IntoOutputExt,
    attack: impl IntoOutputOpt,
    decay: impl IntoOutputOpt,
    sustain: impl IntoOutputOpt,
    release: impl IntoOutputOpt,
) -> Node {
    let pitch = pitch.into_output(graph);
    let gate = gate.into_output(graph);
    let detune = detune.into_output(graph);

    let freq1 = PitchToFreq::default().node(graph, &pitch + &detune);
    let freq2 = PitchToFreq::default().node(graph, &pitch - &detune);

    let saw1 = BlSawOscillator::default().node(graph, freq1);
    let saw2 = BlSawOscillator::default().node(graph, freq2);

    let saws = (saw1 + saw2) * 0.5;
    // let saws = saw1;

    let adsr = Adsr::default().node(graph, gate, attack, decay, sustain, release);
    let adsr = adsr.output(0).scale(20.0, 20_000.0);

    Biquad::lowpass().node(graph, saws, adsr, 20.0, 0.01)
}

fn main() {
    env_logger::init();

    let graph = Graph::new(0, 2);

    let clock = Metro::from_tempo_and_ticks(144.0, 4) // 144bpm, 4 ticks per beat
        .node(&graph, (), ());

    let bd_pat = BoolPattern::default().node(&graph, &clock, "x . . . x . . . x . . . x . . . ");
    let bd = OneShot::load("examples/assets/bd.wav")
        .unwrap()
        .node(&graph, bd_pat.output(0), ());

    let sd_pat = BoolPattern::default().node(&graph, &clock, ". . . . x . . . . . . . x . . x ");
    let sd_vel_pat = Pattern::default().node(&graph, sd_pat.output(0), "2 2 1");
    let sd = OneShot::load("examples/assets/sd.wav")
        .unwrap()
        .node(&graph, sd_pat.output(0), ());
    let sd = &sd.output(0) * &sd_vel_pat.output(0);

    let saw_pat = BoolPattern::default().node(&graph, &clock, "x . x . x . x .");
    let base = 40;
    let saw_notes = Pattern::default().node(&graph, saw_pat.output(0), "0 3 7");
    let saw_notes = &saw_notes.output(0) + base;
    let saw = synth(
        &graph,
        saw_notes.output(0),
        0.0,
        saw_pat.output(0).trig_to_gate(0.1),
        0.0,
        0.12,
        0.0,
        1.0,
    );
    let saw = &saw.output(0) * 0.2;

    let mix = &bd.output(0) + &sd.output(0) + &saw.output(0);
    // let verb = StereoReverb::default().node(&graph, &saw.output(0), &saw.output(0), ());
    // let verb_l = &verb.output(0) * 0.5 + &mix.output(0) * 0.5;
    // let verb_r = &verb[1] * 0.5 + &mix.output(0) * 0.5;

    // let mix_l = verb_l;
    // let mix_r = verb_r;

    let mix_l = mix.clone();
    let mix_r = mix.clone();

    let l = PeakLimiter::default().node(&graph, mix_l, (), (), ());
    let r = PeakLimiter::default().node(&graph, mix_r, (), (), ());

    graph.dac((&l, &r));

    graph.allocate(48000.0, 512);

    let stream = CpalOut::spawn(&AudioBackend::Default, &AudioDevice::Default)
        .record_to_wav("recording.wav");
    graph.play_for(stream, Duration::from_secs(10)).unwrap();
}
