use raug::prelude::ProcResult;
use raug_macros::processor;

#[processor(derive(Clone, Copy, Debug, Default))]
pub fn add_to_counter(
    #[state] counter: &mut f32,
    #[input] a: &f32,
    #[input] b: &f32,
    #[output] out: &mut f32,
) -> ProcResult<()> {
    *counter += a + b;
    *out = *counter;
    Ok(())
}

impl AddToCounter {
    pub fn new(initial_count: f32) -> Self {
        Self {
            counter: initial_count,
            a: 0.0,
            b: 0.0,
        }
    }
}
