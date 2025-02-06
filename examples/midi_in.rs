use raug::prelude::*;

fn main() {
    env_logger::init();
    let graph = GraphBuilder::new();

    let out1 = graph.add_audio_output();
    let out2 = graph.add_audio_output();

    let midi = graph.add_midi_input("midi_in");

    let sine = graph.add(BlSawOscillator::default());

    let freq = graph.add(MidiNote);
    freq.input("midi").connect(midi.output(0));
    let freq = freq.make_register().midi2freq();
    freq.output(0).connect(&sine.input(0));

    let vel = graph.add(MidiVelocity);
    vel.input("midi").connect(midi.output(0));
    let vel = vel.make_register().smooth(0.001);
    let vel = vel / 127.0 * 0.5;

    let mix = sine * vel;

    mix.output(0).connect(&out1.input(0));
    mix.output(0).connect(&out2.input(0));

    let mut runtime = graph.build_runtime();

    raug::util::list_midi_ports();

    let handle = runtime
        .run(
            AudioBackend::Default,
            AudioDevice::Default,
            Some(MidiPort::Name("MPK mini Plus".to_string())), // change this to the name of your MIDI device
        )
        .unwrap();

    std::io::stdin().read_line(&mut String::new()).unwrap();

    handle.stop();
}
