//! Signal types and operations.

use std::{
    fmt::{Debug, Display},
    ops::{
        Add, AddAssign, Deref, DerefMut, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub,
        SubAssign,
    },
    path::Path,
};

use crate::{message::Message, prelude::SignalSpec};

/// A single 64-bit floating-point sample of signal data.
#[derive(Default, Clone, Copy, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Sample(f64);

impl Sample {
    /// The maximum value of a sample.
    pub const MAX: Self = Sample(f64::MAX);
    /// The minimum value of a sample.
    pub const MIN: Self = Sample(f64::MIN);
    /// The value `1.0`.
    pub const ONE: Self = Sample(1.0);
    /// The value `0.0`.
    pub const ZERO: Self = Sample(0.0);
    /// The value `-1.0`.
    pub const NEG_ONE: Self = Sample(-1.0);
    /// The value `e` (Euler's number).
    pub const E: Self = Sample(std::f64::consts::E);
    /// The value `π` (pi).
    pub const PI: Self = Sample(std::f64::consts::PI);
    /// The value `τ` (tau).
    pub const TAU: Self = Sample(std::f64::consts::TAU);
    /// The value `2π`.
    pub const TWO_PI: Self = Sample(std::f64::consts::TAU);

    /// Creates a new sample with the given value.
    #[inline]
    pub const fn new(value: f64) -> Self {
        Sample(value)
    }

    /// Returns the floating-point value of the sample.
    #[inline]
    pub const fn value(self) -> f64 {
        self.0
    }

    /// Returns `true` if the sample is greater than zero.
    #[inline]
    pub fn is_truthy(self) -> bool {
        self.0 > 0.0
    }

    /// Returns `true` if the sample is less than or equal to zero.
    #[inline]
    pub fn is_falsy(self) -> bool {
        self.0 <= 0.0
    }

    /// Returns the absolute value of the sample.
    #[inline]
    pub fn abs(self) -> Self {
        Sample(self.0.abs())
    }

    /// Returns the sine of the value.
    #[inline]
    pub fn sin(self) -> Self {
        Sample(self.0.sin())
    }

    /// Returns the cosine of the value.
    #[inline]
    pub fn cos(self) -> Self {
        Sample(self.0.cos())
    }

    /// Returns the tangent of the value.
    #[inline]
    pub fn tan(self) -> Self {
        Sample(self.0.tan())
    }

    /// Returns the arcsine sine of the value.
    #[inline]
    pub fn asin(self) -> Self {
        Sample(self.0.asin())
    }

    /// Returns the arccosine of the value.
    #[inline]
    pub fn acos(self) -> Self {
        Sample(self.0.acos())
    }

    /// Returns the arctangent of the value.
    #[inline]
    pub fn atan(self) -> Self {
        Sample(self.0.atan())
    }

    /// Returns the arctangent of the ratio of the two values.
    #[inline]
    pub fn atan2(self, other: Self) -> Self {
        Sample(self.0.atan2(other.0))
    }

    /// Converts an amplitude value to a decibel value.
    #[inline]
    pub fn amp_to_db(self) -> Self {
        if self.0 <= 0.0 {
            Self::MIN
        } else {
            Self(20.0 * self.0.log10())
        }
    }

    /// Converts a decibel value to an amplitude value.
    #[inline]
    pub fn db_to_amp(self) -> Self {
        Self(10.0f64.powf(self.0 / 20.0))
    }
}

impl Debug for Sample {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl From<Sample> for f64 {
    #[inline]
    fn from(sample: Sample) -> Self {
        sample.0
    }
}

impl From<f64> for Sample {
    #[inline]
    fn from(value: f64) -> Self {
        Sample(value)
    }
}

impl AsRef<f64> for Sample {
    #[inline]
    fn as_ref(&self) -> &f64 {
        &self.0
    }
}

impl Deref for Sample {
    type Target = f64;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Sample {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Add for Sample {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Sample(self.0 + rhs.0)
    }
}

impl Sub for Sample {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Sample(self.0 - rhs.0)
    }
}

impl Mul for Sample {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        Sample(self.0 * rhs.0)
    }
}

impl Div for Sample {
    type Output = Self;

    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        Sample(self.0 / rhs.0)
    }
}

impl Rem for Sample {
    type Output = Self;

    #[inline]
    fn rem(self, rhs: Self) -> Self::Output {
        Sample(self.0 % rhs.0)
    }
}

impl AddAssign for Sample {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl SubAssign for Sample {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl MulAssign for Sample {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        self.0 *= rhs.0;
    }
}

impl DivAssign for Sample {
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        self.0 /= rhs.0;
    }
}

impl RemAssign for Sample {
    #[inline]
    fn rem_assign(&mut self, rhs: Self) {
        self.0 %= rhs.0;
    }
}

impl Neg for Sample {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        Sample(-self.0)
    }
}

impl Display for Sample {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// An owning, fixed-length array of [`Sample`]s.
/// This type implements [`Deref`] and [`DerefMut`], so it can be indexed and iterated over just like a normal slice.
/// It can also be [`collected`](std::iter::Iterator::collect) from an iterator of [`Sample`]s.
#[derive(PartialEq, Clone)]
pub struct Buffer<T> {
    buf: Vec<T>,
}

impl<T: Debug> Debug for Buffer<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.buf.iter()).finish()
    }
}

impl<T> Buffer<T> {
    /// Creates a new buffer filled with zeros.
    #[inline]
    pub fn zeros(length: usize) -> Self
    where
        T: Default,
    {
        let mut buf = Vec::with_capacity(length);
        for _ in 0..length {
            buf.push(T::default());
        }
        Buffer { buf }
    }

    /// Resizes the buffer to the given length, filling any new elements with the given value.
    #[inline]
    pub fn resize(&mut self, length: usize, value: T)
    where
        T: Clone,
    {
        if self.len() != length {
            self.buf.resize(length, value);
        }
    }

    /// Maps each sample in `other` with `f`, storing the result in the correspeonding sample in `self`.
    #[inline]
    pub fn copy_map<F>(&mut self, other: &[T], mut f: F)
    where
        F: FnMut(&T) -> T,
    {
        for (a, b) in self.buf.iter_mut().zip(other.iter()) {
            *a = f(b);
        }
    }

    /// Iterates over each sample in the buffer, calling `f` with a mutable reference to each sample.
    #[inline]
    pub fn map_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut T),
    {
        for sample in self.buf.iter_mut() {
            f(sample);
        }
    }

    /// Clones the given slice into a new buffer.
    #[inline]
    pub fn from_slice(value: &[T]) -> Self
    where
        T: Clone,
    {
        Buffer {
            buf: value.to_vec(),
        }
    }
}

impl Buffer<Sample> {
    /// Loads a buffer from a WAV file.
    ///
    /// Multi-channel WAV files are supported, but only the first channel will be loaded.
    pub fn load_wav(path: impl AsRef<Path>) -> Result<Self, hound::Error> {
        let reader = hound::WavReader::open(path)?;
        if reader.spec().channels == 1 {
            let samples: Result<Vec<_>, hound::Error> = reader
                .into_samples::<f32>()
                .map(|sample| Ok(Sample(sample? as f64)))
                .collect();
            let samples = samples?;

            Ok(Buffer::from_slice(&samples))
        } else {
            let channels = reader.spec().channels;

            let samples: Result<Vec<_>, hound::Error> = reader
                .into_samples::<f32>()
                .step_by(channels as usize)
                .map(|sample| Ok(Sample(sample? as f64)))
                .collect();
            let samples = samples?;

            Ok(Buffer::from_slice(&samples))
        }
    }

    /// Saves the buffer to a WAV file.
    ///
    /// The buffer will be saved as a single-channel 32-bit WAV file with the given sample rate.
    pub fn save_wav(&self, path: impl AsRef<Path>, sample_rate: u32) -> Result<(), hound::Error> {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        let mut writer = hound::WavWriter::create(path, spec)?;
        for sample in self.buf.iter() {
            writer.write_sample(sample.0 as f32)?;
        }
        writer.finalize()?;
        Ok(())
    }

    /// Returns the maximum value in the buffer.
    #[inline]
    pub fn max(&self) -> Sample {
        self.buf
            .iter()
            .cloned()
            .fold(Sample::MIN, |a, b| a.max(*b).into())
    }

    /// Returns the minimum value in the buffer.
    #[inline]
    pub fn min(&self) -> Sample {
        self.buf
            .iter()
            .cloned()
            .fold(Sample::MAX, |a, b| a.min(*b).into())
    }

    /// Returns the sum of all values in the buffer.
    #[inline]
    pub fn sum(&self) -> Sample {
        self.buf.iter().cloned().fold(Sample::ZERO, |a, b| a + b)
    }

    /// Returns the mean of all values in the buffer.
    #[inline]
    pub fn mean(&self) -> Sample {
        self.sum() / Sample(self.len() as f64)
    }

    /// Returns the root mean square of all values in the buffer.
    #[inline]
    pub fn rms(&self) -> Sample {
        self.buf
            .iter()
            .cloned()
            .fold(Sample::ZERO, |a, b| a + b * b)
    }

    /// Returns the variance of all values in the buffer.
    #[inline]
    pub fn variance(&self) -> Sample {
        let mean = self.mean();
        let sum = self
            .buf
            .iter()
            .cloned()
            .fold(Sample::ZERO, |a, b| a + (b - mean) * (b - mean));
        sum / Sample((self.len() - 1) as f64)
    }

    /// Returns the standard deviation of all values in the buffer.
    #[inline]
    pub fn stddev(&self) -> Sample {
        self.variance().sqrt().into()
    }
}

impl Buffer<Option<Message>> {
    /// Returns `true` if all messages in the buffer are of the same message type.
    pub fn is_homogeneous(&self) -> bool {
        if self.buf.len() > 1 {
            let first_some = self.buf.iter().find(|message| message.is_some());
            if let Some(first_some) = first_some {
                let first_some = first_some.as_ref().unwrap();
                self.buf.iter().all(|message| {
                    message.is_none()
                        || message
                            .as_ref()
                            .is_some_and(|message| message.is_same_type(first_some))
                })
            } else {
                true
            }
        } else {
            true
        }
    }

    /// Panics on debug builds if the buffer is not homogeneous.
    #[track_caller]
    #[inline]
    pub fn debug_assert_homogeneous(&self) {
        debug_assert!(self.is_homogeneous(), "Buffer is not homogeneous");
    }

    /// Returns `true` if all messages in the buffer are `None`.
    #[inline]
    pub fn is_all_none(&self) -> bool {
        self.buf.iter().all(Option::is_none)
    }

    /// Returns `true` if all messages in the buffer are `Some(Message::Bang)`.
    #[inline]
    pub fn is_all_bang(&self) -> bool {
        self.buf.iter().all(|message| {
            message.is_none() || message.as_ref().is_some_and(|message| message.is_bang())
        })
    }

    /// Returns `true` if all messages in the buffer are `Some(Message::Int)`.
    #[inline]
    pub fn is_all_int(&self) -> bool {
        self.buf.iter().all(|message| {
            message.is_none() || message.as_ref().is_some_and(|message| message.is_int())
        })
    }

    /// Returns `true` if all messages in the buffer are `Some(Message::Float)`.
    #[inline]
    pub fn is_all_float(&self) -> bool {
        self.buf.iter().all(|message| {
            message.is_none() || message.as_ref().is_some_and(|message| message.is_float())
        })
    }

    /// Returns `true` if all messages in the buffer are `Some(Message::String)`.
    #[inline]
    pub fn is_all_string(&self) -> bool {
        self.buf.iter().all(|message| {
            message.is_none() || message.as_ref().is_some_and(|message| message.is_string())
        })
    }
}

impl<T> Deref for Buffer<T> {
    type Target = [T];
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.buf.as_ref()
    }
}

impl<T> DerefMut for Buffer<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.buf.as_mut()
    }
}

impl<T> AsRef<[T]> for Buffer<T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.buf.as_ref()
    }
}

impl<T> From<Vec<T>> for Buffer<T> {
    #[inline]
    fn from(vec: Vec<T>) -> Self {
        Buffer { buf: vec }
    }
}

impl<T> From<Box<[T]>> for Buffer<T> {
    #[inline]
    fn from(buf: Box<[T]>) -> Self {
        Buffer {
            buf: buf.into_vec(),
        }
    }
}

impl<T> From<&[T]> for Buffer<T>
where
    T: Clone,
{
    #[inline]
    fn from(slice: &[T]) -> Self {
        Buffer {
            buf: slice.to_vec(),
        }
    }
}

impl<'a, T> IntoIterator for &'a Buffer<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.buf.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Buffer<T> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.buf.iter_mut()
    }
}

/// A signal that can be either a single sample or a message.
#[derive(Debug, Clone)]
pub enum Signal {
    /// A single sample.
    Sample(Sample),
    /// A single message.
    Message(Option<Message>),
}

impl Signal {
    /// Creates a new sample signal with the given value.
    pub const fn new_sample(value: f64) -> Self {
        Self::Sample(Sample(value))
    }

    /// Creates a new message signal with the given message.
    pub fn new_message_some(message: Message) -> Self {
        Self::Message(Some(message))
    }

    /// Creates a new message signal with no message.
    pub fn new_message_none() -> Self {
        Self::Message(None)
    }

    /// Returns `true` if this is a sample.
    pub const fn is_sample(&self) -> bool {
        matches!(self, Self::Sample(_))
    }

    /// Returns `true` if this is a message.
    pub const fn is_message(&self) -> bool {
        matches!(self, Self::Message(_))
    }

    /// Returns the sample value, if this is a sample.
    pub const fn as_sample(&self) -> Option<&Sample> {
        match self {
            Self::Sample(sample) => Some(sample),
            Self::Message(_) => None,
        }
    }

    /// Returns the message value, if this is a message.
    pub const fn as_message(&self) -> Option<&Option<Message>> {
        match self {
            Self::Sample(_) => None,
            Self::Message(message) => Some(message),
        }
    }

    /// Returns the type of signal this is.
    pub const fn kind(&self) -> SignalKind {
        match self {
            Self::Sample(_) => SignalKind::Sample,
            Self::Message(_) => SignalKind::Message,
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<Signal> for Sample {
    fn into(self) -> Signal {
        Signal::Sample(self)
    }
}

#[allow(clippy::from_over_into)]
impl Into<Signal> for f64 {
    fn into(self) -> Signal {
        Signal::Sample(Sample(self))
    }
}

/// A signal kind, which can be either a sample or a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignalKind {
    /// A sample signal.
    Sample,
    /// A message signal.
    Message,
}

/// A buffer that can contain either samples or messages.
#[derive(Debug, Clone)]
pub enum SignalBuffer {
    /// A buffer of samples.
    Sample(Buffer<Sample>),
    /// A buffer of messages.
    Message(Buffer<Option<Message>>),
}

impl SignalBuffer {
    /// Creates a new sample buffer of size `length`, filled with zeros.
    pub fn new_sample(length: usize) -> Self {
        Self::Sample(Buffer::zeros(length))
    }

    /// Creates a new message buffer of size `length`, filled with `None`.
    pub fn new_message(length: usize) -> Self {
        Self::Message(Buffer {
            buf: vec![None; length],
        })
    }

    /// Creates a new buffer from a [`SignalSpec`], filling it with the default value.
    pub fn from_spec_default(spec: &SignalSpec, length: usize) -> Self {
        match &spec.default_value {
            Signal::Sample(default_value) => Self::Sample(Buffer {
                buf: vec![*default_value; length],
            }),
            Signal::Message(mess) => Self::Message(Buffer {
                buf: vec![mess.clone(); length],
            }),
        }
    }

    /// Returns `true` if this is a sample buffer.
    pub fn is_sample(&self) -> bool {
        matches!(self, Self::Sample(_))
    }

    /// Returns `true` if this is a message buffer.
    pub fn is_message(&self) -> bool {
        matches!(self, Self::Message(_))
    }

    /// Returns the signal at the given index.
    #[inline]
    pub fn signal_at(&self, index: usize) -> Option<Signal> {
        match self {
            Self::Sample(buffer) => Some(buffer.buf[index].into()),
            Self::Message(buffer) => Some(Signal::Message(buffer.buf[index].clone())),
        }
    }

    /// Returns a reference to the sample buffer, if this is a sample buffer.
    pub fn as_sample(&self) -> Option<&Buffer<Sample>> {
        match self {
            Self::Sample(buffer) => Some(buffer),
            Self::Message(_) => None,
        }
    }

    /// Returns a reference to the message buffer, if this is a message buffer.
    pub fn as_message(&self) -> Option<&Buffer<Option<Message>>> {
        match self {
            Self::Sample(_) => None,
            Self::Message(buffer) => Some(buffer),
        }
    }

    /// Returns a mutable reference to the sample buffer, if this is a sample buffer.
    pub fn as_sample_mut(&mut self) -> Option<&mut Buffer<Sample>> {
        match self {
            Self::Sample(buffer) => Some(buffer),
            Self::Message(_) => None,
        }
    }

    /// Returns a mutable reference to the message buffer, if this is a message buffer.
    pub fn as_message_mut(&mut self) -> Option<&mut Buffer<Option<Message>>> {
        match self {
            Self::Sample(_) => None,
            Self::Message(buffer) => Some(buffer),
        }
    }

    /// Returns the length of the buffer.
    pub fn len(&self) -> usize {
        match self {
            Self::Sample(buffer) => buffer.len(),
            Self::Message(buffer) => buffer.len(),
        }
    }

    /// Returns `true` if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Sample(buffer) => buffer.is_empty(),
            Self::Message(buffer) => buffer.is_empty(),
        }
    }

    /// Resizes the buffer to the given length, filling any new elements with the given value.
    pub fn resize(&mut self, length: usize, value: Signal) {
        match self {
            Self::Sample(buffer) => {
                if let Signal::Sample(value) = value {
                    buffer.resize(length, value);
                } else {
                    panic!("Cannot resize sample buffer with message");
                }
            }
            Self::Message(buffer) => {
                if let Signal::Message(value) = value {
                    buffer.resize(length, value);
                } else {
                    panic!("Cannot resize message buffer with sample");
                }
            }
        }
    }

    /// Fills the buffer with the given value.
    pub fn fill(&mut self, value: impl Into<Signal>) {
        let value = value.into();
        match self {
            Self::Sample(buffer) => {
                if let Signal::Sample(value) = value {
                    buffer.fill(value);
                } else {
                    panic!("Cannot fill sample buffer with message");
                }
            }
            Self::Message(buffer) => {
                if let Signal::Message(value) = value {
                    buffer.fill(value);
                } else {
                    panic!("Cannot fill message buffer with sample");
                }
            }
        }
    }

    /// Fills the buffer with the default value of the given [`SignalSpec`].
    pub fn fill_with_spec_default(&mut self, spec: &SignalSpec) {
        self.fill(spec.default_value.clone());
    }

    /// Copies the contents of `other` into `self`.
    pub fn copy_from(&mut self, other: &Self) {
        match (self, other) {
            (Self::Sample(this), Self::Sample(other)) => {
                this.copy_from_slice(other);
            }
            (Self::Message(this), Self::Message(other)) => {
                this.clone_from_slice(other);
            }
            _ => panic!("Cannot copy between sample and message buffers"),
        }
    }
}

impl From<SignalBuffer> for Buffer<Signal> {
    fn from(buffer: SignalBuffer) -> Self {
        match buffer {
            SignalBuffer::Sample(buffer) => Buffer {
                buf: buffer.buf.into_iter().map(Signal::Sample).collect(),
            },
            SignalBuffer::Message(buffer) => Buffer {
                buf: buffer.buf.into_iter().map(Signal::Message).collect(),
            },
        }
    }
}

impl TryFrom<Buffer<Signal>> for SignalBuffer {
    type Error = &'static str;

    fn try_from(buffer: Buffer<Signal>) -> Result<Self, Self::Error> {
        let mut sample_buffer = Vec::with_capacity(buffer.len());
        let mut message_buffer = Vec::with_capacity(buffer.len());
        for signal in buffer.buf {
            match signal {
                Signal::Sample(sample) => sample_buffer.push(sample),
                Signal::Message(message) => message_buffer.push(message),
            }
        }
        if !sample_buffer.is_empty() && !message_buffer.is_empty() {
            Err("Buffer contains both samples and messages")
        } else if !sample_buffer.is_empty() {
            Ok(SignalBuffer::Sample(Buffer { buf: sample_buffer }))
        } else {
            Ok(SignalBuffer::Message(Buffer {
                buf: message_buffer,
            }))
        }
    }
}
