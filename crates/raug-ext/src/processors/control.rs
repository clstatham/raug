use raug::prelude::*;

#[processor(derive(Default))]
pub fn cond<T>(
    #[input] condition: &bool,
    #[input] if_true: &T,
    #[input] if_false: &T,
    #[output] out: &mut T,
) -> ProcResult<()>
where
    T: Signal + Default + Clone,
{
    if *condition {
        out.clone_from(if_true);
    } else {
        out.clone_from(if_false);
    }
    Ok(())
}

#[processor(derive(Default))]
pub fn gt<T>(#[input] a: &T, #[input] b: &T, #[output] out: &mut bool) -> ProcResult<()>
where
    T: Signal + PartialOrd + Clone,
{
    *out = a > b;
    Ok(())
}

#[processor(derive(Default))]
pub fn lt<T>(#[input] a: &T, #[input] b: &T, #[output] out: &mut bool) -> ProcResult<()>
where
    T: Signal + PartialOrd + Clone,
{
    *out = a < b;
    Ok(())
}

#[processor(derive(Default))]
pub fn ge<T>(#[input] a: &T, #[input] b: &T, #[output] out: &mut bool) -> ProcResult<()>
where
    T: Signal + PartialOrd + Clone,
{
    *out = a >= b;
    Ok(())
}

#[processor(derive(Default))]
pub fn le<T>(#[input] a: &T, #[input] b: &T, #[output] out: &mut bool) -> ProcResult<()>
where
    T: Signal + PartialOrd + Clone,
{
    *out = a <= b;
    Ok(())
}

#[processor(derive(Default))]
pub fn eq<T>(#[input] a: &T, #[input] b: &T, #[output] out: &mut bool) -> ProcResult<()>
where
    T: Signal + PartialEq + Clone,
{
    *out = a == b;
    Ok(())
}

#[processor(derive(Default))]
pub fn ne<T>(#[input] a: &T, #[input] b: &T, #[output] out: &mut bool) -> ProcResult<()>
where
    T: Signal + PartialEq + Clone,
{
    *out = a != b;
    Ok(())
}
