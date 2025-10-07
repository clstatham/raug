use crate::{Graph, Node};
use wasm_bindgen::prelude::*;

use raug::processor::Processor;

mod raug_ext {
    pub mod builtins {
        pub use raug_ext::processors::*;
    }
}

macro_rules! wrap_processor {
    (mod $module:ident { $($proc:ident $func_name:ident ;)* }) => {
        $(
            #[wasm_bindgen]
            pub struct $proc {
                #[allow(dead_code)]
                pub(crate) inner: $module::builtins::$proc,
            }

            #[wasm_bindgen]
            impl $proc {
                #[wasm_bindgen(constructor)]
                pub fn new() -> Self {
                    Self {
                        inner: <$module::builtins::$proc as Default>::default(),
                    }
                }

                #[wasm_bindgen(js_name = "name")]
                pub fn name(&self) -> String {
                    self.inner.name().to_string()
                }

                #[wasm_bindgen(js_name = "numInputs")]
                pub fn num_inputs(&self) -> u32 {
                    self.inner.input_spec().len() as u32
                }

                #[wasm_bindgen(js_name = "numOutputs")]
                pub fn num_outputs(&self) -> u32 {
                    self.inner.output_spec().len() as u32
                }

                #[wasm_bindgen(js_name = "inputNames")]
                pub fn input_names(&self) -> js_sys::Array {
                    self.inner
                        .input_spec()
                        .iter()
                        .map(|spec| JsValue::from(spec.name.clone()))
                        .collect()
                }

                #[wasm_bindgen(js_name = "outputNames")]
                pub fn output_names(&self) -> js_sys::Array {
                    self.inner
                        .output_spec()
                        .iter()
                        .map(|spec| JsValue::from(spec.name.clone()))
                        .collect()
                }
            }

            impl Default for $proc {
                fn default() -> Self {
                    Self::new()
                }
            }

            #[wasm_bindgen]
            impl Graph {
                #[allow(non_snake_case)]
                #[wasm_bindgen(js_name = $func_name)]
                pub fn $func_name(&mut self) -> Node {
                    Node {
                        inner: self.inner.node(<$module::builtins::$proc as Default>::default()),
                    }
                }
            }
        )*

    };
}

macro_rules! wrap_processor_generic {
    (mod $module:ident { $($proc:ident $func_name:ident = $inner_proc:ident < $t:ty >;)* }) => {
        $(
            #[wasm_bindgen]
            pub struct $proc {
                #[allow(dead_code)]
                pub(crate) inner: $module::builtins::$inner_proc<$t>,
            }

            #[wasm_bindgen]
            impl $proc {
                #[wasm_bindgen(constructor)]
                pub fn new() -> Self {
                    Self {
                        inner: <$module::builtins::$inner_proc<$t> as Default>::default(),
                    }
                }

                #[wasm_bindgen(js_name = "name")]
                pub fn name(&self) -> String {
                    self.inner.name().to_string()
                }
            }

            impl Default for $proc {
                fn default() -> Self {
                    Self::new()
                }
            }

            #[wasm_bindgen]
            impl Graph {
                #[allow(non_snake_case)]
                #[wasm_bindgen(js_name = $func_name)]
                pub fn $func_name(&mut self) -> Node {
                    Node {
                        inner: self.inner.node(<$module::builtins::$inner_proc<$t> as Default>::default()),
                    }
                }
            }
        )*
    };
}

wrap_processor! {
    mod raug {
        Add add;
        Sub sub;
        Mul mul;
        Div div;
        Rem rem;
        Neg neg;
    }
}

wrap_processor! {
    mod raug_ext {
        Powf powf;
        Sqrt sqrt;
        Sin sin;
        Cos cos;
        Tan tan;
        Asin asin;
        Acos acos;
        Atan atan;
        Sinh sinh;
        Cosh cosh;
        Tanh tanh;
        Atan2 atan2;
        Hypot hypot;
        Abs abs;
        Ceil ceil;
        Floor floor;
        Round round;
        Trunc trunc;
        Fract fract;
        Recip recip;
        Signum signum;
        Exp exp;
        Exp2 exp2;
        Log log;
        Log2 log2;
        Log10 log10;
        SineOscillator sineOscillator;
        BlSawOscillator blSawOscillator;
        NoiseOscillator noiseOscillator;
        PhaseAccumulator phaseAccumulator;
        Smooth smooth;
    }
}

wrap_processor_generic! {
    mod raug {
        ConstantFloat constantFloat = Constant<f32>;
        ConstantBool constantBool = Constant<bool>;
    }
}

wrap_processor_generic! {
    mod raug_ext {
        FloatMax floatMax = Max<f32>;
        FloatMin floatMin = Min<f32>;
        FloatClamp floatClamp = Clamp<f32>;
    }
}
