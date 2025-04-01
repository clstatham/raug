use std::{
    fmt::Debug,
    num::{NonZeroU32, NonZeroU64},
};

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct FloatRepr32(NonZeroU32);

impl FloatRepr32 {
    pub const NICHE: NonZeroU32 = NonZeroU32::MAX;

    #[inline]
    pub fn new(value: f32) -> Self {
        if value.to_bits() == 0 {
            Self(Self::NICHE)
        } else {
            Self(unsafe { NonZeroU32::new_unchecked(value.to_bits() ^ Self::NICHE.get()) })
        }
    }

    #[inline]
    pub fn get(self) -> f32 {
        if self.0.get() == Self::NICHE.get() {
            f32::from_bits(0)
        } else {
            f32::from_bits(self.0.get() ^ Self::NICHE.get())
        }
    }
}

impl Debug for FloatRepr32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get())
    }
}

impl PartialEq for FloatRepr32 {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get()
    }
}

impl Default for FloatRepr32 {
    #[inline]
    fn default() -> Self {
        FloatRepr32::new(0.0)
    }
}

impl From<f32> for FloatRepr32 {
    #[inline]
    fn from(value: f32) -> Self {
        FloatRepr32::new(value)
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct FloatRepr64(NonZeroU64);

impl FloatRepr64 {
    const NICHE: NonZeroU64 = NonZeroU64::MAX;

    #[inline]
    pub fn new(value: f64) -> Self {
        if value.to_bits() == 0 {
            Self(Self::NICHE)
        } else {
            Self(unsafe { NonZeroU64::new_unchecked(value.to_bits() ^ Self::NICHE.get()) })
        }
    }

    #[inline]
    pub fn get(self) -> f64 {
        if self.0.get() == Self::NICHE.get() {
            f64::from_bits(0)
        } else {
            f64::from_bits(self.0.get() ^ Self::NICHE.get())
        }
    }
}

impl Debug for FloatRepr64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get())
    }
}

impl PartialEq for FloatRepr64 {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get()
    }
}

impl Default for FloatRepr64 {
    #[inline]
    fn default() -> Self {
        FloatRepr64::new(0.0)
    }
}

impl From<f64> for FloatRepr64 {
    #[inline]
    fn from(value: f64) -> Self {
        FloatRepr64::new(value)
    }
}

#[cfg(feature = "f32_samples")]
pub type FloatRepr = FloatRepr32;
#[cfg(not(feature = "f32_samples"))]
pub type FloatRepr = FloatRepr64;

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
