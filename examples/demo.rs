use raug::prelude::*;

#[processor]
pub fn sine_oscillator(
    env: ProcEnv,
    #[state] phase: &mut f32,
    #[input] freq: &f32,
    #[output] out: &mut f32,
) -> ProcResult<()> {
    *phase += 2.0 * std::f32::consts::PI * freq / env.sample_rate;
    *out = phase.sin() * 0.2;
    Ok(())
}

fn main() {
    // initialize logging
    env_logger::init();

    // create a new graph
    let graph = Graph::new();

    // add some outputs
    let out1 = graph.add_audio_output();
    let out2 = graph.add_audio_output();

    // add a sine oscillator
    let sine = graph.add(SineOscillator {
        phase: 0.0,
        freq: 440.0,
        out: 0.0,
    });

    // connect the sine oscillator to the outputs
    sine.output(0).connect(&out1.input(0));
    sine.output(0).connect(&out2.input(0));

    // open the audio stream
    let mut stream = CpalStream::default();
    // run the graph
    stream.spawn(&graph).unwrap();
    stream.play().unwrap();
    std::thread::sleep(std::time::Duration::from_secs(60));
    stream.stop().unwrap();
}
