//! Utility processors.

use crate::prelude::*;

/// A processor that does nothing.
///
/// This is used for audio inputs to the graph, since a buffer will be allocated for it, which will be filled by the audio backend.
#[processor(derive(Default))]
pub(crate) fn null(#[output] _out: &mut f32) -> ProcResult<()> {
    Ok(())
}

/// A processor that passes its input to its output unchanged.
#[processor(derive(Default))]
pub fn passthrough<T>(#[input] input: &T, #[output] out: &mut T) -> ProcResult<()>
where
    T: Signal + Default + Clone,
{
    out.clone_from(input);
    Ok(())
}
