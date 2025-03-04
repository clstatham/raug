//! Mathematical processors.

use crate::{prelude::*, processor::ProcessorError, signal::AnySignalOptMut};
use std::ops::{
    Add as AddOp, Div as DivOp, Mul as MulOp, Neg as NegOp, Rem as RemOp, Sub as SubOp,
};

/// A processor that outputs a constant value every sample.
///
/// # Inputs
///
/// None.
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Any` | The constant value. |
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Constant {
    value: AnySignal,
}

impl Constant {
    /// Creates a new `Constant` processor that outputs the given `Signal`.
    pub fn new(value: impl Signal) -> Self {
        Self::new_any(value.into_any_signal())
    }

    /// Creates a new `Constant` processor that outputs the given `AnySignal`.
    pub fn new_any(value: AnySignal) -> Self {
        Self { value }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for Constant {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", self.value.signal_type())]
    }

    fn process(
        &mut self,
        _inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        outputs.output(0).fill(self.value.into_any_signal_opt());

        Ok(())
    }
}

impl GraphBuilder {
    /// Adds a node that outputs a constant value every sample.
    pub fn constant(&self, value: impl Signal + Clone) -> Node {
        self.add(Constant::new(value))
    }
}

/// A processor that converts MIDI note numbers to frequencies.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `note` | `Float` | The MIDI note number. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `freq` | `Float` | The frequency of the MIDI note. |
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MidiToFreq;

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for MidiToFreq {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("note", SignalType::Float)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("freq", SignalType::Float)]
    }

    #[allow(clippy::useless_conversion)]
    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (note, freq) in raug_macros::iter_proc_io_as!(inputs as [Float], outputs as [Float]) {
            if let Some(note) = note.into_option() {
                freq.set((2.0 as Float).powf((note - 69.0) / 12.0) * 440.0);
            } else {
                freq.set_none();
            }
        }

        Ok(())
    }
}

/// A processor that converts frequencies to MIDI note numbers.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `freq` | `Float` | The frequency to convert. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `note` | `Float` | The MIDI note number. |
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FreqToMidi;

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for FreqToMidi {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("freq", SignalType::Float)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("note", SignalType::Float)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (freq, note) in raug_macros::iter_proc_io_as!(inputs as [Float], outputs as [Float]) {
            if let Some(freq) = freq.into_option() {
                note.set(69.0 + 12.0 * (freq / 440.0).log2());
            } else {
                note.set_none();
            }
        }

        Ok(())
    }
}

macro_rules! impl_binary_proc {
    ($name:ident, $method:ident, ($($data:ident = $ty:ty),*), $doc:literal) => {
        #[derive(Clone, Debug)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[doc = $doc]
        pub struct $name {
            a: AnySignalOpt,
            b: AnySignalOpt,
        }

        impl $name {
            #[doc = concat!("Creates a new `", stringify!($name), "` processor.")]
            pub fn new(signal_type: SignalType) -> Self {
                Self {
                    a: AnySignalOpt::default_of_type(&signal_type),
                    b: AnySignalOpt::default_of_type(&signal_type),
                }
            }
        }

        #[cfg_attr(feature = "serde", typetag::serde)]
        impl Processor for $name {
            fn input_spec(&self) -> Vec<SignalSpec> {
                vec![
                    SignalSpec::new("a", self.a.signal_type()),
                    SignalSpec::new("b", self.b.signal_type()),
                ]
            }

            fn output_spec(&self) -> Vec<SignalSpec> {
                vec![SignalSpec::new("out", self.a.signal_type())]
            }

            fn process(
                &mut self,
                inputs: ProcessorInputs,
                outputs: ProcessorOutputs,
            ) -> Result<(), ProcessorError> {
                for (in1, in2, mut out) in raug_macros::iter_proc_io_as!(inputs as [Any, Any], outputs as [Any]) {
                    if let Some(in1) = in1 {
                        if in1.signal_type() != self.a.signal_type() {
                            return Err(ProcessorError::InputSpecMismatch {
                                index: 0,
                                expected: self.a.signal_type(),
                                actual: in1.signal_type(),
                            });
                        }
                        self.a = in1;
                    } else {
                        out.set_none();
                        return Ok(());
                    }

                    if let Some(in2) = in2 {
                        if in2.signal_type() != self.b.signal_type() {
                            return Err(ProcessorError::InputSpecMismatch {
                                index: 1,
                                expected: self.b.signal_type(),
                                actual: in2.signal_type(),
                            });
                        }
                        self.b = in2;
                    } else {
                        out.set_none();
                        return Ok(());
                    }

                    match out {
                        $(AnySignalOptMut::$data(sample) => {
                            let a = self.a.as_type::<$ty>().unwrap_or_default();
                            let b = self.b.as_type::<$ty>().unwrap_or_default();
                            sample.set(a.$method(b));
                        })*
                        sample => {
                            return Err(ProcessorError::OutputSpecMismatch {
                                index: 0,
                                expected: self.a.signal_type(),
                                actual: sample.signal_type(),
                            });
                        }
                    }
                }


                Ok(())
            }
        }
    };
}

impl_binary_proc!(
    Add,
    add,
    (Float = Float, Int = i64),
    "A processor that adds two signals together."
);
impl_binary_proc!(
    Sub,
    sub,
    (Float = Float, Int = i64),
    "A processor that subtracts one signal from another."
);
impl_binary_proc!(
    Mul,
    mul,
    (Float = Float, Int = i64),
    "A processor that multiplies two signals together."
);
impl_binary_proc!(
    Div,
    div,
    (Float = Float, Int = i64),
    "A processor that divides one signal by another."
);
impl_binary_proc!(
    Rem,
    rem,
    (Float = Float, Int = i64),
    "A processor that calculates the remainder of dividing one signal by another."
);
impl_binary_proc!(
    Powf,
    powf,
    (Float = Float),
    "A processor that raises one signal to the power of another."
);
impl_binary_proc!(
    Atan2,
    atan2,
    (Float = Float),
    "A processor that calculates the arctangent of the ratio of two signals."
);
impl_binary_proc!(
    Hypot,
    hypot,
    (Float = Float),
    "A processor that calculates the hypotenuse of two signals."
);
impl_binary_proc!(
    Max,
    max,
    (Float = Float, Int = i64),
    "A processor that calculates the maximum of two signals."
);
impl_binary_proc!(
    Min,
    min,
    (Float = Float, Int = i64),
    "A processor that calculates the minimum of two signals."
);

macro_rules! impl_unary_proc {
    ($name:ident, $method:ident, ($($data:ident = $ty:ty),*), $doc:literal) => {
        #[derive(Clone, Debug)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[doc = $doc]
        pub struct $name {
            a: AnySignalOpt,
        }

        impl $name {
            #[doc = concat!("Creates a new `", stringify!($name), "` processor.")]
            pub fn new(signal_type: SignalType) -> Self {
                Self {
                    a: AnySignalOpt::default_of_type(&signal_type),
                }
            }
        }

        #[cfg_attr(feature = "serde", typetag::serde)]
        impl Processor for $name {
            fn input_spec(&self) -> Vec<SignalSpec> {
                vec![SignalSpec::new("a", self.a.signal_type())]
            }

            fn output_spec(&self) -> Vec<SignalSpec> {
                vec![SignalSpec::new("out", self.a.signal_type())]
            }

            fn process(
                &mut self,
                inputs: ProcessorInputs,
                outputs: ProcessorOutputs,
            ) -> Result<(), ProcessorError> {
                for (a, mut out) in raug_macros::iter_proc_io_as!(inputs as [Any], outputs as [Any]) {
                    if let Some(a) = a {
                        if a.signal_type() != self.a.signal_type() {
                            return Err(ProcessorError::InputSpecMismatch {
                                index: 0,
                                expected: self.a.signal_type(),
                                actual: a.signal_type(),
                            });
                        }
                        self.a = a;
                    } else {
                        out.set_none();
                        return Ok(());
                    }

                    match out {
                        $(AnySignalOptMut::$data(sample) => {
                            let a = self.a.as_type::<$ty>().unwrap();
                            sample.set(a.$method());
                        })*
                        sample => {
                            return Err(ProcessorError::OutputSpecMismatch {
                                index: 0,
                                expected: self.a.signal_type(),
                                actual: sample.signal_type(),
                            });
                        }
                    }
                }


                Ok(())
            }
        }
    };
}

impl_unary_proc!(
    Neg,
    neg,
    (Float = Float, Int = i64),
    "A processor that negates a signal."
);
impl_unary_proc!(
    Abs,
    abs,
    (Float = Float, Int = i64),
    "A processor that calculates the absolute value of a signal."
);
impl_unary_proc!(
    Sqrt,
    sqrt,
    (Float = Float),
    "A processor that calculates the square root of a signal."
);
impl_unary_proc!(
    Cbrt,
    cbrt,
    (Float = Float),
    "A processor that calculates the cube root of a signal."
);
impl_unary_proc!(
    Ceil,
    ceil,
    (Float = Float),
    "A processor that rounds a signal up to the nearest integer."
);
impl_unary_proc!(
    Floor,
    floor,
    (Float = Float),
    "A processor that rounds a signal down to the nearest integer."
);
impl_unary_proc!(
    Round,
    round,
    (Float = Float),
    "A processor that rounds a signal to the nearest integer."
);
impl_unary_proc!(
    Trunc,
    trunc,
    (Float = Float),
    "A processor that truncates a signal to an integer."
);
impl_unary_proc!(
    Fract,
    fract,
    (Float = Float),
    "A processor that outputs the fractional part of a signal."
);
impl_unary_proc!(
    Recip,
    recip,
    (Float = Float),
    "A processor that calculates the reciprocal of a signal."
);
impl_unary_proc!(
    Signum,
    signum,
    (Float = Float, Int = i64),
    "A processor that outputs the sign of a signal."
);
impl_unary_proc!(
    Sin,
    sin,
    (Float = Float),
    "A processor that calculates the sine of a signal."
);
impl_unary_proc!(
    Cos,
    cos,
    (Float = Float),
    "A processor that calculates the cosine of a signal."
);
impl_unary_proc!(
    Tan,
    tan,
    (Float = Float),
    "A processor that calculates the tangent of a signal."
);
impl_unary_proc!(
    Tanh,
    tanh,
    (Float = Float),
    "A processor that calculates the hyperbolic tangent of a signal."
);
impl_unary_proc!(
    Exp,
    exp,
    (Float = Float),
    "A processor that calculates the natural exponential of a signal."
);
impl_unary_proc!(
    Ln,
    ln,
    (Float = Float),
    "A processor that calculates the natural logarithm of a signal."
);
impl_unary_proc!(
    Log2,
    log2,
    (Float = Float),
    "A processor that calculates the base-2 logarithm of a signal."
);
impl_unary_proc!(
    Log10,
    log10,
    (Float = Float),
    "A processor that calculates the base-10 logarithm of a signal."
);
