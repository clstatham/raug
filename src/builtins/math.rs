//! Math built-in processors.

use std::marker::PhantomData;

use crate::prelude::*;

/// A processor that outputs a constant value.
#[processor(derive(Default))]
pub fn constant<T>(#[state] value: &mut T, #[output] out: &mut T) -> ProcResult<()>
where
    T: Signal + Default + Clone,
{
    out.clone_from(value);

    Ok(())
}

impl<T: Signal + Default + Clone> Constant<T> {
    /// Creates a new constant processor with the given value.
    pub fn new(value: T) -> Self {
        Self {
            value,
            _marker0: PhantomData,
        }
    }
}

/// A processor that adds two values and outputs the result.
#[processor(derive(Default))]
pub fn add<T>(#[input] a: &T, #[input] b: &T, #[output] out: &mut T) -> ProcResult<()>
where
    T: Signal + Default + Clone,
    for<'a> &'a T: std::ops::Add<Output = T>,
{
    *out = a + b;
    Ok(())
}

/// A processor that subtracts two values and outputs the result.
#[processor(derive(Default))]
pub fn sub<T>(#[input] a: &T, #[input] b: &T, #[output] out: &mut T) -> ProcResult<()>
where
    T: Signal + Default + Clone,
    for<'a> &'a T: std::ops::Sub<Output = T>,
{
    *out = a - b;
    Ok(())
}

/// A processor that multiplies two values and outputs the result.
#[processor(derive(Default))]
pub fn mul<T>(#[input] a: &T, #[input] b: &T, #[output] out: &mut T) -> ProcResult<()>
where
    T: Signal + Default + Clone,
    for<'a> &'a T: std::ops::Mul<Output = T>,
{
    *out = a * b;
    Ok(())
}

/// A processor that divides two values and outputs the result.
#[processor(derive(Default))]
pub fn div<T>(#[input] a: &T, #[input] b: &T, #[output] out: &mut T) -> ProcResult<()>
where
    T: Signal + Default + Clone,
    for<'a> &'a T: std::ops::Div<Output = T>,
{
    *out = a / b;
    Ok(())
}

/// A processor that computes the modulus of two values and outputs the result.
#[processor(derive(Default))]
pub fn rem<T>(#[input] a: &T, #[input] b: &T, #[output] out: &mut T) -> ProcResult<()>
where
    T: Signal + Default + Clone,
    for<'a> &'a T: std::ops::Rem<Output = T>,
{
    *out = a % b;
    Ok(())
}

/// A processor that computes the absolute value of a value and outputs the result.
#[processor(derive(Default))]
pub fn neg<T>(#[input] a: &T, #[output] out: &mut T) -> ProcResult<()>
where
    T: Signal + Default + Clone,
    for<'a> &'a T: std::ops::Neg<Output = T>,
{
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
