use crate::prelude::*;

/// A processor that passes a real signal through unchanged.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `RealBuf` | The input signal. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `RealBuf` | The output signal. |
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RealBufPassthrough(pub FftBufLength);

#[cfg_attr(feature = "serde", typetag::serde)]
impl FftProcessor for RealBufPassthrough {
    fn input_spec(&self) -> Vec<FftSpec> {
        vec![FftSpec::new("in", FftSignalType::RealBuf(self.0))]
    }

    fn output_spec(&self) -> Vec<FftSpec> {
        vec![FftSpec::new("out", FftSignalType::RealBuf(self.0))]
    }

    fn process(
        &mut self,
        _fft_length: usize,
        inputs: &[&FftSignal],
        outputs: &mut [FftSignal],
    ) -> Result<(), ProcessorError> {
        let in_signal = inputs[0].as_real_buf().unwrap();
        let out_signal = outputs[0].as_real_buf_mut().unwrap();
        out_signal.copy_from_slice(in_signal);
        Ok(())
    }
}

/// A processor that generates a constant real signal.
///
/// # Inputs
///
/// None.
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `RealBuf` | The constant signal. |
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RealSplat {
    value: f32,
    len: FftBufLength,
}

impl RealSplat {
    /// Creates a new [`RealBufSplat`] processor with the given FFT buffer length and value.
    pub fn new(value: f32, len: FftBufLength) -> Self {
        Self { len, value }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl FftProcessor for RealSplat {
    fn input_spec(&self) -> Vec<FftSpec> {
        vec![]
    }

    fn output_spec(&self) -> Vec<FftSpec> {
        vec![FftSpec::new("out", FftSignalType::RealBuf(self.len))]
    }

    fn process(
        &mut self,
        _fft_length: usize,
        _inputs: &[&FftSignal],
        outputs: &mut [FftSignal],
    ) -> Result<(), ProcessorError> {
        let out_signal = outputs[0].as_real_buf_mut().unwrap();
        out_signal.fill(self.value);
        Ok(())
    }
}

/// A processor that accumulates a real signal.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `RealBuf` | The input signal. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `RealBuf` | The accumulated signal. |
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RealBufAccumulator {
    len: FftBufLength,
    accum: RealBuf,
}

impl RealBufAccumulator {
    /// Creates a new [`RealBufAccumulator`] processor with the given FFT buffer length.
    pub fn new(len: FftBufLength) -> Self {
        Self {
            len,
            accum: RealBuf::default(),
        }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl FftProcessor for RealBufAccumulator {
    fn input_spec(&self) -> Vec<FftSpec> {
        vec![FftSpec::new("in", FftSignalType::RealBuf(self.len))]
    }

    fn output_spec(&self) -> Vec<FftSpec> {
        vec![FftSpec::new("out", FftSignalType::RealBuf(self.len))]
    }

    fn allocate(&mut self, fft_length: usize, _padded_length: usize) {
        self.accum = vec![0.0; self.len.calculate(fft_length, 0)] // todo: add block_size input to this function
            .into_iter()
            .collect();
    }

    fn process(
        &mut self,
        _fft_length: usize,
        inputs: &[&FftSignal],
        outputs: &mut [FftSignal],
    ) -> Result<(), ProcessorError> {
        let in_signal = inputs[0].as_real_buf().unwrap();
        let out_signal = outputs[0].as_real_buf_mut().unwrap();
        for (out, input, accum) in itertools::izip!(
            out_signal.iter_mut(),
            in_signal.iter(),
            self.accum.iter_mut()
        ) {
            *accum += *input;
            *out = *accum;
        }
        Ok(())
    }
}

/// A processor that wraps a real signal to +/- a given value.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `RealBuf` | The input signal. |
/// | `1` | `wrap` | `RealBuf` | The value to wrap around. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `RealBuf` | The wrapped signal. |
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RealBufWrap(pub FftBufLength);

#[cfg_attr(feature = "serde", typetag::serde)]
impl FftProcessor for RealBufWrap {
    fn input_spec(&self) -> Vec<FftSpec> {
        vec![
            FftSpec::new("in", FftSignalType::RealBuf(self.0)),
            FftSpec::new("wrap", FftSignalType::RealBuf(self.0)),
        ]
    }

    fn output_spec(&self) -> Vec<FftSpec> {
        vec![FftSpec::new("out", FftSignalType::RealBuf(self.0))]
    }

    fn process(
        &mut self,
        _fft_length: usize,
        inputs: &[&FftSignal],
        outputs: &mut [FftSignal],
    ) -> Result<(), ProcessorError> {
        let in_signal = inputs[0].as_real_buf().unwrap();
        let wrap_signal = inputs[1].as_real_buf().unwrap();
        let out_signal = outputs[0].as_real_buf_mut().unwrap();
        for (out, input, wrap) in
            itertools::izip!(out_signal.iter_mut(), in_signal.iter(), wrap_signal.iter())
        {
            let wrap = wrap.abs();
            if wrap == 0.0 {
                *out = 0.0;
                continue;
            }
            *out = *input;

            while *out < -wrap {
                *out += wrap * 2.0;
            }
            while *out >= wrap {
                *out -= wrap * 2.0;
            }
        }
        Ok(())
    }
}

macro_rules! real_binary_op {
    ($name:ident, $doc:literal, $op:tt) => {

        #[derive(Debug, Clone)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[doc = concat!($doc, r#"\n
# Inputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `a` | `RealBuf` | The first signal. |
| `1` | `b` | `RealBuf` | The second signal. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `RealBuf` | The result of the operation. |"#)]
        pub struct $name(pub FftBufLength);

        #[cfg_attr(feature = "serde", typetag::serde)]
        impl FftProcessor for $name {
            fn input_spec(&self) -> Vec<FftSpec> {
                vec![
                    FftSpec::new("a", FftSignalType::RealBuf(self.0)),
                    FftSpec::new("b", FftSignalType::RealBuf(self.0)),
                ]
            }

            fn output_spec(&self) -> Vec<FftSpec> {
                vec![FftSpec::new("out", FftSignalType::RealBuf(self.0))]
            }

            fn process(
                &mut self,
                _fft_length: usize,
                inputs: &[&FftSignal],
                outputs: &mut [FftSignal],
            ) -> Result<(), ProcessorError> {
                let a = inputs[0].as_real_buf().unwrap();
                let b = inputs[1].as_real_buf().unwrap();
                let out = outputs[0].as_real_buf_mut().unwrap();
                for (out, a, b) in itertools::izip!(out.iter_mut(), a.iter(), b.iter()) {
                    *out = a $op b;
                }

                Ok(())
            }
        }
    };
}

real_binary_op!(
    RealAdd,
    "A processor that adds two real signals.",
     +
);

real_binary_op!(
    RealSub,
    "A processor that subtracts two real signals.",
     -
);

real_binary_op!(
    RealMul,
    "A processor that multiplies two real signals.",
     *
);

real_binary_op!(
    RealDiv,
    "A processor that divides two real signals.",
     /
);

real_binary_op!(
    RealRem,
    "A processor that computes the remainder of two real signals.",
     %
);
