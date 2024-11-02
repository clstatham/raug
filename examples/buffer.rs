use daprs::prelude::*;

fn main() {
    // initialize logging
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // create a new graph
    let graph = GraphBuilder::new();

    // add some outputs
    let out1 = graph.output();
    let out2 = graph.output();

    // add a buffer reader
    let buf = Buffer::load_wav("examples/assets/piano1.wav").unwrap();
    let len = buf.len() as f64;
    let buffer = graph.buffer_reader(buf);

    // connect the buffer reader to the outputs
    buffer.connect_output(0, out1, 0);
    buffer.connect_output(0, out2, 0);

    // create a sawtooth oscillator to drive the buffer reader
    let saw = graph.saw_osc();

    // we want to read the sample to its full length, so set the frequency to the sample rate divided by the length
    let freq = graph.sample_rate() / len;
    freq.connect_output(0, saw, "frequency");

    // multiply the saw oscillator's amplitude by the length of the buffer, so it outputs the full range of the buffer
    let saw = saw * len;

    // convert the saw oscillator to output an integer message
    let saw = saw.s2m().f2i();

    // connect the saw oscillator to the buffer reader
    saw.connect_output(0, buffer, "t");

    // build the graph
    let graph = graph.build();

    // create a new runtime
    let mut runtime = Runtime::new(graph);

    // run the runtime for 5 seconds and output to a file
    runtime
        .run_offline_to_file("target/buffer.wav", Duration::from_secs(5), 48_000.0, 512)
        .unwrap();
}
