use std::{
    fmt::Debug,
    num::{NonZeroU32, NonZeroU64},
};

pub trait Nullable: Sized {
    const NULL: Self;

    fn is_null(&self) -> bool;

    #[inline]
    fn is_not_null(&self) -> bool {
        !self.is_null()
    }
}

pub trait Optional<T: Sized>: Nullable {
    #[inline]
    fn none() -> Self {
        Self::NULL
    }
    #[inline]
    fn some(value: T) -> Self {
        Self::from_option(Some(value))
    }

    #[inline]
    fn is_none(&self) -> bool {
        self.is_null()
    }

    #[inline]
    fn is_some(&self) -> bool {
        self.is_not_null()
    }

    #[inline]
    fn unwrap(self) -> T {
        self.into_option().unwrap()
    }

    #[inline]
    fn unwrap_or(self, default: T) -> T {
        self.into_option().unwrap_or(default)
    }

    #[inline]
    fn unwrap_or_default(self) -> T
    where
        T: Default,
    {
        self.into_option().unwrap_or_default()
    }

    #[inline]
    fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Option<U> {
        self.into_option().map(f)
    }

    fn into_option(self) -> Option<T>;
    fn from_option(option: Option<T>) -> Self;

    #[inline]
    fn set_none(&mut self) {
        *self = Self::NULL;
    }

    #[inline]
    fn set(&mut self, value: T) -> Self {
        std::mem::replace(self, Self::some(value))
    }
}

impl<T> Nullable for Option<T> {
    const NULL: Self = None;

    #[inline]
    fn is_null(&self) -> bool {
        self.is_none()
    }
}

impl<T> Optional<T> for Option<T> {
    #[inline]
    fn into_option(self) -> Option<T> {
        self
    }

    #[inline]
    fn from_option(option: Option<T>) -> Self {
        option
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct OptFloat32(NonZeroU32);

impl OptFloat32 {
    pub const NICHE: u32 = u32::MAX;

    #[inline]
    pub fn new(value: f32) -> Self {
        if value.to_bits() == Self::NICHE {
            Self::NULL
        } else {
            Self(unsafe { NonZeroU32::new_unchecked(value.to_bits() ^ Self::NICHE) })
        }
    }
}

impl Nullable for OptFloat32 {
    const NULL: Self = Self(unsafe { NonZeroU32::new_unchecked(Self::NICHE) });

    #[inline]
    fn is_null(&self) -> bool {
        self.0 == Self::NULL.0
    }
}

impl Optional<f32> for OptFloat32 {
    #[inline]
    fn into_option(self) -> Option<f32> {
        if self.is_null() {
            None
        } else {
            Some(f32::from_bits(self.0.get() ^ Self::NICHE))
        }
    }

    #[inline]
    fn from_option(option: Option<f32>) -> Self {
        match option {
            Some(value) => Self::new(value),
            None => Self::NULL,
        }
    }
}

impl Debug for OptFloat32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_null() {
            write!(f, "None")
        } else {
            write!(f, "Some({})", self.into_option().unwrap())
        }
    }
}

impl PartialEq for OptFloat32 {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.into_option() == other.into_option()
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct OptFloat64(NonZeroU64);

impl OptFloat64 {
    pub const NICHE: u64 = u64::MAX;

    #[inline]
    pub fn new(value: f64) -> Self {
        if value.to_bits() == Self::NICHE {
            Self::NULL
        } else {
            Self(unsafe { NonZeroU64::new_unchecked(value.to_bits() ^ Self::NICHE) })
        }
    }
}

impl Nullable for OptFloat64 {
    const NULL: Self = Self(unsafe { NonZeroU64::new_unchecked(Self::NICHE) });

    #[inline]
    fn is_null(&self) -> bool {
        self.0 == Self::NULL.0
    }
}

impl Optional<f64> for OptFloat64 {
    #[inline]
    fn into_option(self) -> Option<f64> {
        if self.is_null() {
            None
        } else {
            Some(f64::from_bits(self.0.get() ^ Self::NICHE))
        }
    }

    #[inline]
    fn from_option(option: Option<f64>) -> Self {
        match option {
            Some(value) => Self::new(value),
            None => Self::NULL,
        }
    }
}

impl Debug for OptFloat64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_null() {
            write!(f, "None")
        } else {
            write!(f, "Some({})", self.into_option().unwrap())
        }
    }
}

impl PartialEq for OptFloat64 {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.into_option() == other.into_option()
    }
}

#[cfg(feature = "f32_samples")]
pub type OptFloat = OptFloat32;
#[cfg(not(feature = "f32_samples"))]
pub type OptFloat = OptFloat64;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opt_float32() {
        let value = OptFloat32::new(1.0);
        assert_eq!(value.into_option(), Some(1.0));

        let value = OptFloat32::NULL;
        assert_eq!(value.into_option(), None);

        let value = OptFloat32::from_option(Some(1.0));
        assert_eq!(value.into_option(), Some(1.0));

        let value = OptFloat32::from_option(None);
        assert_eq!(value.into_option(), None);

        assert_eq!(size_of::<OptFloat32>(), size_of::<f32>());
    }

    #[test]
    fn test_opt_float64() {
        let value = OptFloat64::new(1.0);
        assert_eq!(value.into_option(), Some(1.0));

        let value = OptFloat64::NULL;
        assert_eq!(value.into_option(), None);

        let value = OptFloat64::from_option(Some(1.0));
        assert_eq!(value.into_option(), Some(1.0));

        let value = OptFloat64::from_option(None);
        assert_eq!(value.into_option(), None);

        assert_eq!(size_of::<OptFloat64>(), size_of::<f64>());
    }
}
