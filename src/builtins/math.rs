use crate::prelude::*;

#[processor(derive(Default))]
pub fn constant<T>(#[state] value: &mut T, #[output] out: &mut T) -> ProcResult<()>
where
    T: Signal,
{
    *out = *value;
    Ok(())
}

impl<T: Signal> Constant<T> {
    pub fn new(value: T) -> Self {
        Self { value, out: value }
    }
}

#[processor(derive(Default))]
pub fn add<T>(#[input] a: &T, #[input] b: &T, #[output] out: &mut T) -> ProcResult<()>
where
    T: std::ops::Add<Output = T> + Signal,
{
    *out = *a + *b;
    Ok(())
}

#[processor(derive(Default))]
pub fn sub<T>(#[input] a: &T, #[input] b: &T, #[output] out: &mut T) -> ProcResult<()>
where
    T: std::ops::Sub<Output = T> + Signal,
{
    *out = *a - *b;
    Ok(())
}

#[processor(derive(Default))]
pub fn mul<T>(#[input] a: &T, #[input] b: &T, #[output] out: &mut T) -> ProcResult<()>
where
    T: std::ops::Mul<Output = T> + Signal,
{
    *out = *a * *b;
    Ok(())
}

#[processor(derive(Default))]
pub fn div<T>(#[input] a: &T, #[input] b: &T, #[output] out: &mut T) -> ProcResult<()>
where
    T: std::ops::Div<Output = T> + Signal,
{
    *out = *a / *b;
    Ok(())
}

#[processor(derive(Default))]
pub fn rem<T>(#[input] a: &T, #[input] b: &T, #[output] out: &mut T) -> ProcResult<()>
where
    T: std::ops::Rem<Output = T> + Signal,
{
    *out = *a % *b;
    Ok(())
}

#[processor(derive(Default))]
pub fn neg<T>(#[input] a: &T, #[output] out: &mut T) -> ProcResult<()>
where
    T: std::ops::Neg<Output = T> + Signal,
{
    *out = -*a;
    Ok(())
}
