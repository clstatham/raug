use std::{fmt::Debug, num::NonZeroU32};

use super::Signal;

pub trait Repr<T: Signal> {
    fn from_signal(value: T) -> Self;
    fn into_signal(self) -> T;
}

impl<T: Signal> Repr<T> for T {
    #[inline]
    fn from_signal(value: T) -> Self {
        value
    }

    #[inline]
    fn into_signal(self) -> T {
        self
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct FloatRepr(NonZeroU32);

impl FloatRepr {
    pub const NICHE: NonZeroU32 = NonZeroU32::MAX;

    #[inline]
    pub fn new(value: f32) -> Self {
        let bits = value.to_bits() ^ Self::NICHE.get();
        Self(NonZeroU32::new(bits).map_or(Self::NICHE, |n| n))
    }

    #[inline]
    pub const fn get(self) -> f32 {
        let bits = self.0.get() ^ Self::NICHE.get();
        f32::from_bits(bits)
    }
}

impl Debug for FloatRepr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get())
    }
}

impl PartialEq for FloatRepr {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get()
    }
}

impl Default for FloatRepr {
    #[inline]
    fn default() -> Self {
        FloatRepr::new(0.0)
    }
}

impl From<f32> for FloatRepr {
    #[inline]
    fn from(value: f32) -> Self {
        FloatRepr::new(value)
    }
}

impl Repr<f32> for FloatRepr {
    #[inline]
    fn from_signal(value: f32) -> Self {
        FloatRepr::new(value)
    }

    #[inline]
    fn into_signal(self) -> f32 {
        self.get()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_float_repr() {
        assert_eq!(size_of::<Option<FloatRepr>>(), size_of::<FloatRepr>());
        let value = 1.0;
        let repr = FloatRepr::new(value);
        assert_eq!(repr.get(), value);
    }
}
