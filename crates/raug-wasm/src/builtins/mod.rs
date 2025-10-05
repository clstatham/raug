use crate::{Graph, Node};
use wasm_bindgen::prelude::*;

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

/*

pub trait GraphExt {
    fn powf<A, AO, B, BO>(&mut self, a: A, b: B) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        BO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>,
        B: IntoNodeOutput<BO>;

    fn sqrt<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>;

    fn sin<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>;

    fn cos<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>;

    fn tan<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>;

    fn asin<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>;

    fn acos<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>;

    fn atan<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>;

    fn sinh<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>;

    fn cosh<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>;

    fn tanh<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>;

    fn atan2<A, AO, B, BO>(&mut self, a: A, b: B) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        BO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>,
        B: IntoNodeOutput<BO>;

    fn hypot<A, AO, B, BO>(&mut self, a: A, b: B) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        BO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>,
        B: IntoNodeOutput<BO>;

    fn abs<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>;

    fn ceil<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>;

    fn floor<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>;

    fn round<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>;

    fn trunc<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>;

    fn fract<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>;

    fn recip<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>;

    fn signum<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>;

    fn max<A, AO, B, BO>(&mut self, a: A, b: B) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        BO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>,
        B: IntoNodeOutput<BO>;

    fn min<A, AO, B, BO>(&mut self, a: A, b: B) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        BO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>,
        B: IntoNodeOutput<BO>;

    fn clamp<A, AO, B, BO, C, CO>(&mut self, a: A, min: B, max: C) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        BO: AsNodeOutputIndex<ProcessorNode>,
        CO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>,
        B: IntoNodeOutput<BO>,
        C: IntoNodeOutput<CO>;
}
 */
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
        SineOscillator sineOscillator;
        BlSawOscillator blSawOscillator;

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
