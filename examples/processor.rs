use raug::prelude::*;

#[derive(Debug, Clone, Processor)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", processor_typetag)]
struct GainProc {
    gain: Float,

    #[input]
    input: Float,
    #[output]
    out: Float,
}

impl GainProc {
    pub fn new(gain: Float) -> Self {
        Self {
            gain,
            input: 0.0,
            out: 0.0,
        }
    }

    fn update(&mut self, _env: &ProcEnv) {
        self.out = self.input * self.gain;
    }
}

fn main() {
    env_logger::init();

    let graph = GraphBuilder::new();

    let out1 = graph.add_audio_output();
    let out2 = graph.add_audio_output();

    let sine = graph.add(SineOscillator::default());
    sine.input("frequency").connect(440.0);

    let gain = graph.add(GainProc::new(0.5));

    sine.output(0).connect(&gain.input(0));

    gain.output(0).connect(&out1.input(0));
    gain.output(0).connect(&out2.input(0));

    let mut runtime = graph.build_runtime();

    runtime
        .run_for(
            Duration::from_secs(1),
            AudioBackend::Default,
            AudioDevice::Default,
            None,
        )
        .unwrap();
}
