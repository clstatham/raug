use std::{fmt::Debug, num::NonZeroU32};

use super::Signal;

/// An internal trait for types that can be converted to and from a `Signal`.
pub trait Repr<T: Signal>
where
    Self: From<T>,
    T: From<Self>,
{
}

impl<T: Signal, U> Repr<T> for U
where
    U: From<T>,
    T: From<U>,
{
}

/// A niche-optimized representation of a 32-bit float.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct NicheFloat(NonZeroU32);

impl NicheFloat {
    // the bits of u32::MAX are in NaN space for 32 bit floats, so it's unlikely to show up in practice
    const NICHE: NonZeroU32 = NonZeroU32::MAX;

    #[inline]
    pub fn new(value: f32) -> Self {
        let bits = value.to_bits() ^ Self::NICHE.get();
        Self(NonZeroU32::new(bits).unwrap_or(Self::NICHE))
    }

    #[inline]
    pub const fn get(self) -> f32 {
        let bits = self.0.get() ^ Self::NICHE.get();
        f32::from_bits(bits)
    }
}

impl Debug for NicheFloat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get())
    }
}

impl PartialEq for NicheFloat {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get()
    }
}

impl Default for NicheFloat {
    #[inline]
    fn default() -> Self {
        NicheFloat::new(0.0)
    }
}

impl From<f32> for NicheFloat {
    #[inline]
    fn from(value: f32) -> Self {
        NicheFloat::new(value)
    }
}

impl From<NicheFloat> for f32 {
    #[inline]
    fn from(value: NicheFloat) -> Self {
        value.get()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_float_repr() {
        assert_eq!(size_of::<Option<NicheFloat>>(), size_of::<NicheFloat>());
        let value = 1.0;
        let repr = NicheFloat::new(value);
        assert_eq!(repr.get(), value);
    }
}
