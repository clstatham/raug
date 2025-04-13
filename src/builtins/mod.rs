//! Built-in processors and utilities for the audio graph.
#![allow(unused_imports)]

pub mod control;
pub mod dynamics;
pub mod filters;
pub mod math;
pub mod oscillators;
pub mod storage;
pub mod time;
pub mod util;

pub use control::*;
pub use dynamics::*;
pub use filters::*;
pub use math::*;
pub use oscillators::*;
pub use storage::*;
pub use time::*;
pub use util::*;

/// Linear interpolation.
#[doc(hidden)]
#[inline]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    debug_assert!((0.0..=1.0).contains(&t));
    a + (b - a) * t
}
