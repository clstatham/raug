use std::sync::Arc;

use num::Complex;

use crate::{fft::FftError, prelude::*};

/// A processor that performs a real-to-complex FFT.
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
/// | `0` | `out` | `ComplexBuf` | The output signal. |
#[derive(Clone)]
pub struct Rfft {
    padded_length: usize,
    plan: Arc<dyn realfft::RealToComplex<f32>>,
    scratch: ComplexBuf,
    in_signal_copy: RealBuf,
}

impl Rfft {
    /// Creates a new `RealFft` processor with the given FFT window length.
    pub fn new(padded_length: usize) -> Self {
        let mut planner = realfft::RealFftPlanner::new();
        let plan = planner.plan_fft_forward(padded_length);
        let scratch = plan.make_scratch_vec().into_boxed_slice();
        let in_signal_copy = plan.make_input_vec().into_boxed_slice();
        Self {
            padded_length,
            plan,
            scratch: ComplexBuf(scratch),
            in_signal_copy: RealBuf(in_signal_copy),
        }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl FftProcessor for Rfft {
    fn input_spec(&self) -> Vec<FftSpec> {
        vec![FftSpec::new(
            "in",
            FftSignalType::RealBuf(FftBufLength::PaddedLength),
        )]
    }

    fn output_spec(&self) -> Vec<FftSpec> {
        vec![FftSpec::new(
            "out",
            FftSignalType::ComplexBuf(FftBufLength::FftLengthPlusOne),
        )]
    }

    fn process(
        &mut self,
        _fft_length: usize,
        inputs: &[&FftSignal],
        outputs: &mut [FftSignal],
    ) -> Result<(), ProcessorError> {
        let in_signal = inputs[0].as_real_buf().unwrap();
        let out_signal = outputs[0].as_complex_buf_mut().unwrap();

        self.in_signal_copy.copy_from_slice(in_signal);

        let res = self.plan.process_with_scratch(
            self.in_signal_copy.as_mut(),
            out_signal.as_mut(),
            self.scratch.as_mut(),
        );

        self.in_signal_copy.fill(0.0);
        self.scratch.fill(Complex::default());

        if let Err(e) = res {
            return Err(ProcessorError::Fft(FftError::RealFft(e.to_string())));
        }

        Ok(())
    }
}

/// A processor that performs a complex-to-real inverse FFT.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `ComplexBuf` | The input signal. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `RealBuf` | The output signal. |
#[derive(Clone)]
pub struct Irfft {
    padded_length: usize,
    plan: Arc<dyn realfft::ComplexToReal<f32>>,
    scratch: ComplexBuf,
    in_signal_copy: ComplexBuf,
}

impl Irfft {
    /// Creates a new `Irfft` processor with the given FFT window length.
    pub fn new(padded_length: usize) -> Self {
        let mut planner = realfft::RealFftPlanner::new();
        let plan = planner.plan_fft_inverse(padded_length);
        let scratch = plan.make_scratch_vec().into_boxed_slice();
        let in_signal_copy = plan.make_input_vec().into_boxed_slice();
        Self {
            padded_length,
            plan,
            scratch: ComplexBuf(scratch),
            in_signal_copy: ComplexBuf(in_signal_copy),
        }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl FftProcessor for Irfft {
    fn input_spec(&self) -> Vec<FftSpec> {
        vec![FftSpec::new(
            "in",
            FftSignalType::ComplexBuf(FftBufLength::FftLengthPlusOne),
        )]
    }

    fn output_spec(&self) -> Vec<FftSpec> {
        vec![FftSpec::new(
            "out",
            FftSignalType::RealBuf(FftBufLength::PaddedLength),
        )]
    }

    fn process(
        &mut self,
        _fft_length: usize,
        inputs: &[&FftSignal],
        outputs: &mut [FftSignal],
    ) -> Result<(), ProcessorError> {
        let in_signal = inputs[0].as_complex_buf().unwrap();
        let out_signal = outputs[0].as_real_buf_mut().unwrap();

        self.in_signal_copy.copy_from_slice(in_signal);

        self.in_signal_copy[0].im = 0.0;
        self.in_signal_copy[self.padded_length / 2].im = 0.0;

        let res = self.plan.process_with_scratch(
            self.in_signal_copy.as_mut(),
            out_signal.as_mut(),
            self.scratch.as_mut(),
        );

        self.in_signal_copy.fill(Complex::default());
        self.scratch.fill(Complex::default());

        if let Err(e) = res {
            return Err(ProcessorError::Fft(FftError::RealFft(e.to_string())));
        }

        Ok(())
    }
}

/// A utility that outputs a buffer of real numbers counting from 0 to the FFT length.
///
/// # Inputs
///
/// None.
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `RealBuf` | The output buffer. |
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BinNumber;

#[cfg_attr(feature = "serde", typetag::serde)]
impl FftProcessor for BinNumber {
    fn input_spec(&self) -> Vec<FftSpec> {
        vec![]
    }

    fn output_spec(&self) -> Vec<FftSpec> {
        vec![FftSpec::new(
            "out",
            FftSignalType::RealBuf(FftBufLength::FftLengthPlusOne),
        )]
    }

    fn process(
        &mut self,
        _fft_length: usize,
        _: &[&FftSignal],
        outputs: &mut [FftSignal],
    ) -> Result<(), ProcessorError> {
        let [output] = outputs else {
            return Err(ProcessorError::NumOutputsMismatch);
        };

        let output = output.as_real_buf_mut().unwrap();

        for (i, x) in output.iter_mut().enumerate() {
            *x = i as f32;
        }

        Ok(())
    }
}

/// Prints the input signal to the console.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `Any` | The input signal. |
///
/// # Outputs
///
/// None.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FftPrint(pub FftSignalType);

#[cfg_attr(feature = "serde", typetag::serde)]
impl FftProcessor for FftPrint {
    fn input_spec(&self) -> Vec<FftSpec> {
        vec![FftSpec::new("in", self.0)]
    }

    fn output_spec(&self) -> Vec<FftSpec> {
        vec![]
    }

    fn process(
        &mut self,
        _fft_length: usize,
        inputs: &[&FftSignal],
        _: &mut [FftSignal],
    ) -> Result<(), ProcessorError> {
        let in_signal = inputs[0];
        println!("{:?}", in_signal);
        Ok(())
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use super::*;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Serialize, Deserialize)]
    struct RfftSerde {
        padded_length: usize,
    }

    impl Serialize for Rfft {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            RfftSerde {
                padded_length: self.padded_length,
            }
            .serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for Rfft {
        fn deserialize<D>(deserializer: D) -> Result<Rfft, D::Error>
        where
            D: Deserializer<'de>,
        {
            let RfftSerde { padded_length } = RfftSerde::deserialize(deserializer)?;
            Ok(Rfft::new(padded_length))
        }
    }

    #[derive(Serialize, Deserialize)]
    struct IrfftSerde {
        padded_length: usize,
    }

    impl Serialize for Irfft {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            IrfftSerde {
                padded_length: self.padded_length,
            }
            .serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for Irfft {
        fn deserialize<D>(deserializer: D) -> Result<Irfft, D::Error>
        where
            D: Deserializer<'de>,
        {
            let IrfftSerde { padded_length } = IrfftSerde::deserialize(deserializer)?;
            Ok(Irfft::new(padded_length))
        }
    }
}
