//! Math built-in processors.

use std::{marker::PhantomData, sync::Arc};

use crossbeam::atomic::AtomicCell;

use crate::prelude::*;

/// A processor that outputs a constant value.
pub struct Constant<T: Signal + Clone> {
    value: T,
}

impl<T: Signal + Default + Clone> Default for Constant<T> {
    fn default() -> Self {
        Self {
            value: T::default(),
        }
    }
}

impl<T: Signal + Clone> Constant<T> {
    /// Creates a new constant processor with the given value.
    pub fn new(value: T) -> Self {
        Self { value }
    }
}

impl<T: Signal + Clone> Processor for Constant<T> {
    fn name(&self) -> &str {
        "Constant"
    }

    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", T::signal_type())]
    }

    fn create_output_buffers(&self, size: usize) -> Vec<AnyBuffer> {
        vec![AnyBuffer::full::<T>(size, self.value.clone())]
    }

    fn process(
        &mut self,
        _inputs: ProcessorInputs,
        _outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        // constants never change, and the buffer is already pre-filled with the constant value
        Ok(())
    }
}

#[processor(derive(Default, Clone))]
pub fn param<T>(#[state] value: &mut Arc<AtomicCell<T>>, #[output] out: &mut T) -> ProcResult<()>
where
    T: Signal + Copy + Default,
{
    *out = value.load();
    Ok(())
}

impl<T> Param<T>
where
    T: Signal + Copy + Default,
{
    pub fn new(init: T) -> Self {
        Param {
            value: Arc::new(AtomicCell::new(init)),
            _t: PhantomData,
        }
    }

    pub fn set(&self, value: T) {
        self.value.store(value);
    }

    pub fn get(&self) -> T {
        self.value.load()
    }
}

/// A processor that adds two values and outputs the result.
#[processor(derive(Default))]
pub fn add(#[input] a: &f32, #[input] b: &f32, #[output] out: &mut f32) -> ProcResult<()> {
    *out = a + b;
    Ok(())
}

/// A processor that subtracts two values and outputs the result.
#[processor(derive(Default))]
pub fn sub(#[input] a: &f32, #[input] b: &f32, #[output] out: &mut f32) -> ProcResult<()> {
    *out = a - b;
    Ok(())
}

/// A processor that multiplies two values and outputs the result.
#[processor(derive(Default))]
pub fn mul(#[input] a: &f32, #[input] b: &f32, #[output] out: &mut f32) -> ProcResult<()> {
    *out = a * b;
    Ok(())
}

/// A processor that divides two values and outputs the result.
#[processor(derive(Default))]
pub fn div(#[input] a: &f32, #[input] b: &f32, #[output] out: &mut f32) -> ProcResult<()> {
    *out = a / b;
    Ok(())
}

/// A processor that computes the modulus of two values and outputs the result.
#[processor(derive(Default))]
pub fn rem(#[input] a: &f32, #[input] b: &f32, #[output] out: &mut f32) -> ProcResult<()> {
    *out = a % b;
    Ok(())
}

/// A processor that computes the absolute value of a value and outputs the result.
#[processor(derive(Default))]
pub fn neg(#[input] a: &f32, #[output] out: &mut f32) -> ProcResult<()> {
    *out = -a;
    Ok(())
}

#[processor(derive(Default))]
pub fn and(#[input] a: &bool, #[input] b: &bool, #[output] out: &mut bool) -> ProcResult<()> {
    *out = *a && *b;
    Ok(())
}

#[processor(derive(Default))]
pub fn or(#[input] a: &bool, #[input] b: &bool, #[output] out: &mut bool) -> ProcResult<()> {
    *out = *a || *b;
    Ok(())
}

#[processor(derive(Default))]
pub fn not(#[input] a: &bool, #[output] out: &mut bool) -> ProcResult<()> {
    *out = !*a;
    Ok(())
}

#[processor(derive(Default))]
pub fn xor(#[input] a: &bool, #[input] b: &bool, #[output] out: &mut bool) -> ProcResult<()> {
    *out = a ^ b;
    Ok(())
}
