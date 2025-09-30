use std::cmp::PartialOrd;

use raug::prelude::*;

#[processor(derive(Default))]
pub fn powf(#[input] a: &f32, #[input] b: &f32, #[output] out: &mut f32) -> ProcResult<()> {
    *out = a.powf(*b);
    Ok(())
}

#[processor(derive(Default))]
pub fn sqrt(#[input] a: &f32, #[output] out: &mut f32) -> ProcResult<()> {
    *out = a.sqrt();
    Ok(())
}

#[processor(derive(Default))]
pub fn cbrt(#[input] a: &f32, #[output] out: &mut f32) -> ProcResult<()> {
    *out = a.cbrt();
    Ok(())
}

#[processor(derive(Default))]
pub fn exp(#[input] a: &f32, #[output] out: &mut f32) -> ProcResult<()> {
    *out = a.exp();
    Ok(())
}

#[processor(derive(Default))]
pub fn exp2(#[input] a: &f32, #[output] out: &mut f32) -> ProcResult<()> {
    *out = a.exp2();
    Ok(())
}

#[processor(derive(Default))]
pub fn ln(#[input] a: &f32, #[output] out: &mut f32) -> ProcResult<()> {
    *out = a.ln();
    Ok(())
}

#[processor(derive(Default))]
pub fn log(#[input] a: &f32, #[input] b: &f32, #[output] out: &mut f32) -> ProcResult<()> {
    *out = a.log(*b);
    Ok(())
}

#[processor(derive(Default))]
pub fn log2(#[input] a: &f32, #[output] out: &mut f32) -> ProcResult<()> {
    *out = a.log2();
    Ok(())
}

#[processor(derive(Default))]
pub fn log10(#[input] a: &f32, #[output] out: &mut f32) -> ProcResult<()> {
    *out = a.log10();
    Ok(())
}

#[processor(derive(Default))]
pub fn sin(#[input] a: &f32, #[output] out: &mut f32) -> ProcResult<()> {
    *out = a.sin();
    Ok(())
}

#[processor(derive(Default))]
pub fn cos(#[input] a: &f32, #[output] out: &mut f32) -> ProcResult<()> {
    *out = a.cos();
    Ok(())
}

#[processor(derive(Default))]
pub fn tan(#[input] a: &f32, #[output] out: &mut f32) -> ProcResult<()> {
    *out = a.tan();
    Ok(())
}

#[processor(derive(Default))]
pub fn asin(#[input] a: &f32, #[output] out: &mut f32) -> ProcResult<()> {
    *out = a.asin();
    Ok(())
}

#[processor(derive(Default))]
pub fn acos(#[input] a: &f32, #[output] out: &mut f32) -> ProcResult<()> {
    *out = a.acos();
    Ok(())
}

#[processor(derive(Default))]
pub fn atan(#[input] a: &f32, #[output] out: &mut f32) -> ProcResult<()> {
    *out = a.atan();
    Ok(())
}

#[processor(derive(Default))]
pub fn atan2(#[input] a: &f32, #[input] b: &f32, #[output] out: &mut f32) -> ProcResult<()> {
    *out = a.atan2(*b);
    Ok(())
}

#[processor(derive(Default))]
pub fn hypot(#[input] a: &f32, #[input] b: &f32, #[output] out: &mut f32) -> ProcResult<()> {
    *out = a.hypot(*b);
    Ok(())
}

#[processor(derive(Default))]
pub fn sinh(#[input] a: &f32, #[output] out: &mut f32) -> ProcResult<()> {
    *out = a.sinh();
    Ok(())
}

#[processor(derive(Default))]
pub fn cosh(#[input] a: &f32, #[output] out: &mut f32) -> ProcResult<()> {
    *out = a.cosh();
    Ok(())
}

#[processor(derive(Default))]
pub fn tanh(#[input] a: &f32, #[output] out: &mut f32) -> ProcResult<()> {
    *out = a.tanh();
    Ok(())
}

#[processor(derive(Default))]
pub fn asinh(#[input] a: &f32, #[output] out: &mut f32) -> ProcResult<()> {
    *out = a.asinh();
    Ok(())
}

#[processor(derive(Default))]
pub fn acosh(#[input] a: &f32, #[output] out: &mut f32) -> ProcResult<()> {
    *out = a.acosh();
    Ok(())
}

#[processor(derive(Default))]
pub fn atanh(#[input] a: &f32, #[output] out: &mut f32) -> ProcResult<()> {
    *out = a.atanh();
    Ok(())
}

#[processor(derive(Default))]
pub fn abs(#[input] a: &f32, #[output] out: &mut f32) -> ProcResult<()> {
    *out = a.abs();
    Ok(())
}

#[processor(derive(Default))]
pub fn signum(#[input] a: &f32, #[output] out: &mut f32) -> ProcResult<()> {
    *out = a.signum();
    Ok(())
}

#[processor(derive(Default))]
pub fn floor(#[input] a: &f32, #[output] out: &mut f32) -> ProcResult<()> {
    *out = a.floor();
    Ok(())
}

#[processor(derive(Default))]
pub fn ceil(#[input] a: &f32, #[output] out: &mut f32) -> ProcResult<()> {
    *out = a.ceil();
    Ok(())
}

#[processor(derive(Default))]
pub fn round(#[input] a: &f32, #[output] out: &mut f32) -> ProcResult<()> {
    *out = a.round();
    Ok(())
}

#[processor(derive(Default))]
pub fn trunc(#[input] a: &f32, #[output] out: &mut f32) -> ProcResult<()> {
    *out = a.trunc();
    Ok(())
}

#[processor(derive(Default))]
pub fn fract(#[input] a: &f32, #[output] out: &mut f32) -> ProcResult<()> {
    *out = a.fract();
    Ok(())
}

#[processor(derive(Default))]
pub fn recip(#[input] a: &f32, #[output] out: &mut f32) -> ProcResult<()> {
    *out = a.recip();
    Ok(())
}

#[processor(derive(Default))]
pub fn max<T>(#[input] a: &T, #[input] b: &T, #[output] out: &mut T) -> ProcResult<()>
where
    T: PartialOrd + Signal + Default + Clone,
{
    out.clone_from(if a > b { a } else { b });
    Ok(())
}

#[processor(derive(Default))]
pub fn min<T>(#[input] a: &T, #[input] b: &T, #[output] out: &mut T) -> ProcResult<()>
where
    T: PartialOrd + Signal + Default + Clone,
{
    out.clone_from(if a < b { a } else { b });
    Ok(())
}

#[processor(derive(Default))]
pub fn clamp<T>(
    #[input] a: &T,
    #[input] min: &T,
    #[input] max: &T,
    #[output] out: &mut T,
) -> ProcResult<()>
where
    T: PartialOrd + Signal + Default + Clone,
{
    out.clone_from(if a < min {
        min
    } else if a > max {
        max
    } else {
        a
    });
    Ok(())
}

#[processor(derive(Default))]
pub fn lerp(
    #[input] a: &f32,
    #[input] b: &f32,
    #[input] t: &f32,
    #[output] out: &mut f32,
) -> ProcResult<()> {
    *out = *a + (*b - *a) * *t;
    Ok(())
}

#[processor(derive(Default))]
pub fn smooth(
    #[state] x: &mut f32,
    #[input] input: &f32,
    #[input] factor: &f32,
    #[output] out: &mut f32,
) -> ProcResult<()> {
    let factor = factor.clamp(0.0, 1.0);
    *x = *x + (*input - *x) * factor;
    *out = *x;

    Ok(())
}

#[processor(derive(Default))]
pub fn pitch_to_freq(#[input] pitch: &f32, #[output] freq: &mut f32) -> ProcResult<()> {
    *freq = 440.0f32 * 2.0f32.powf((*pitch - 69.0f32) / 12.0);
    Ok(())
}

#[processor(derive(Default))]
pub fn scale(
    #[input] t: &f32,
    #[input] min: &f32,
    #[input] max: &f32,
    #[output] out: &mut f32,
) -> ProcResult<()> {
    *out = t * (max - min) + min;
    Ok(())
}
