//! Utility processors.

use crate::prelude::*;

/// A processor that does nothing.
///
/// This is used for audio inputs to the graph, since a buffer will be allocated for it, which will be filled by the audio backend.
#[processor(derive(Default))]
pub(crate) fn null(#[output] _out: &mut f32) {}

/// A processor that passes its input to its output unchanged.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `input` | `T` | The input signal.
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `T` | The output signal.
#[processor(derive(Default))]
pub fn passthrough<T: Signal>(#[input] input: &T, #[output] out: &mut T) {
    *out = *input;
}
