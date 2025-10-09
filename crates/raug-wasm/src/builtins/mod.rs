use raug_macros::wrap_wasm;
use wasm_bindgen::prelude::*;

use raug::processor::Processor;

mod raug_ext {
    pub mod builtins {
        pub use raug_ext::processors::*;
    }
}

#[wasm_bindgen]
pub struct Proc {
    pub(crate) inner: Box<dyn Processor>,
}

#[wasm_bindgen]
pub struct ProcFactory;

#[wasm_bindgen]
impl ProcFactory {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self
    }
}

impl Default for ProcFactory {
    fn default() -> Self {
        Self::new()
    }
}

wrap_wasm! {
    mod raug {
        Add, Sub, Mul, Div, Rem, Neg, Constant<f32>, Constant<bool>, And, Or, Not, Xor,
    }
    mod raug_ext {
        // control.rs
        Cond<f32>, Gt<f32>, Lt<f32>, Ge<f32>, Le<f32>, Eq<f32>, Ne<f32>,
        // math.rs
        Powf, Sqrt, Sin, Cos, Tan, Asin, Acos, Atan, Sinh, Cosh, Tanh, Atan2, Hypot,
        Smooth, PitchToFreq, FreqToPitch,
        Max<f32>, Min<f32>, Clamp<f32>,
        Abs, Ceil, Floor, Round, Trunc, Fract, Recip, Signum, Exp, Exp2, Log, Log2,
        Log10,
        // oscillators.rs
        SineOscillator, BlSawOscillator, NoiseOscillator, PhaseAccumulator,
        // dynamics.rs
        PeakLimiter,
        // filters.rs
        Lowpass1, Highpass1, Biquad,
        // list.rs
        Get<f32>, Get<bool>,
        // storage.rs
        Sample, OneShot,
        // time.rs
        Metro, Decay, Adsr, BoolPattern, Pattern, Delay, Allpass, MonoReverb, StereoReverb,
        // util.rs
        SampleRate, Message<f32>, Message<bool>, Register<f32>, Register<bool>,
        SampleAndHold<f32>, SampleAndHold<bool>,
        UnwrapOr<f32>, UnwrapOr<bool>, Some<f32>, Some<bool>,
        RandomChoice<f32>, RandomChoice<bool>,
    }
}

impl Biquad {
    pub fn lowpass() -> Self {
        Self {
            inner: raug_ext::builtins::Biquad::lowpass(),
        }
    }

    pub fn highpass() -> Self {
        Self {
            inner: raug_ext::builtins::Biquad::highpass(),
        }
    }

    pub fn bandpass() -> Self {
        Self {
            inner: raug_ext::builtins::Biquad::bandpass(),
        }
    }

    pub fn notch() -> Self {
        Self {
            inner: raug_ext::builtins::Biquad::notch(),
        }
    }

    pub fn allpass() -> Self {
        Self {
            inner: raug_ext::builtins::Biquad::allpass(),
        }
    }

    pub fn lowshelf() -> Self {
        Self {
            inner: raug_ext::builtins::Biquad::lowshelf(),
        }
    }

    pub fn highshelf() -> Self {
        Self {
            inner: raug_ext::builtins::Biquad::highshelf(),
        }
    }

    pub fn peaking() -> Self {
        Self {
            inner: raug_ext::builtins::Biquad::peaking(),
        }
    }
}
