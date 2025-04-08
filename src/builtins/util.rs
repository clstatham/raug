//! Utility processors.

use std::sync::{Arc, Mutex};

use crossbeam_channel::{Receiver, Sender};

use crate::prelude::*;

use super::lerp;

/// A processor that does nothing.
///
/// This is used for audio inputs to the graph, since a buffer will be allocated for it, which will be filled by the audio backend.
///
/// # Inputs
///
/// None.
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Float` | The output signal. |
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Null;

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for Null {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalType::Float)]
    }

    fn process(
        &mut self,
        _: ProcessorInputs,
        _outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        Ok(())
    }
}

/// A processor that passes its input to its output unchanged.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `Any` | The input signal.
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Any` | The output signal.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Passthrough {
    signal_type: SignalType,
}

impl Passthrough {
    /// Create a new `Passthrough` processor.
    pub fn new(signal_type: SignalType) -> Self {
        Self { signal_type }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for Passthrough {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("in", self.signal_type)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", self.signal_type)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (input, mut output) in iter_proc_io_as!(inputs as [Any], outputs as [Any]) {
            if let Some(input) = input {
                output.set_any_opt(input);
            }
        }

        Ok(())
    }
}

/// A processor that casts its input to a different signal type.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `S` | The input signal.
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `T` | The output signal.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cast {
    from: SignalType,
    to: SignalType,
}

impl Cast {
    /// Create a new `Cast` processor.
    pub fn new(from: SignalType, to: SignalType) -> Self {
        Self { from, to }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for Cast {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("in", self.from)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", self.to)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (input, mut output) in iter_proc_io_as!(inputs as [Any], outputs as [Any]) {
            if let Some(input) = input {
                if let Some(signal) = input.try_into_any_signal() {
                    if let Some(cast) = signal.to_owned().cast(self.to) {
                        output.set_any(cast);
                    } else {
                        return Err(ProcessorError::InvalidCast(signal.signal_type(), self.to));
                    }
                }
            }
        }

        Ok(())
    }
}

/// A processor that outputs a signal when triggered.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `trig` | `Bool` | The trigger signal. |
/// | `1` | `in` | `Any` | The signal to output. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Any` | The output signal. |
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Message {
    message: AnySignalOpt,
}

impl Message {
    /// Create a new `MessageSender` processor with the given message.
    pub fn new(message: impl Signal) -> Self {
        Self::new_any(message.into_any_signal().into_any_signal_opt())
    }

    /// Create a new `MessageSender` processor with the given message.
    pub fn new_any(message: AnySignalOpt) -> Self {
        Self { message }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for Message {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("trig", SignalType::Bool),
            SignalSpec::new("message", self.message.signal_type()),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", self.message.signal_type())]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (trig, message, mut output) in iter_proc_io_as!(inputs as [bool, Any], outputs as [Any])
        {
            if trig.unwrap_or(false) {
                if let Some(message) = message {
                    if message.signal_type() != self.message.signal_type() {
                        return Err(ProcessorError::InputSpecMismatch {
                            index: 1,
                            expected: self.message.signal_type(),
                            actual: message.signal_type(),
                        });
                    }
                    self.message = message;
                }
                output.set_any_opt(self.message);
            }
        }

        Ok(())
    }
}

/// A processor that prints a signal to the console when triggered.
///
/// The signal will be cast to a string before printing.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `trig` | `Bool` | The trigger signal. |
/// | `1` | `message` | `Any` | The message to print. |
///
/// # Outputs
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Print {
    msg: AnySignalOpt,
}

impl Print {
    /// Create a new `Print` processor with the given message.
    pub fn with_message(message: impl Signal) -> Self {
        Self {
            msg: message.into_any_signal().into_any_signal_opt(),
        }
    }

    /// Create a new `Print` processor with an empty message.
    pub fn new(signal_type: SignalType) -> Self {
        Self {
            msg: AnySignalOpt::default_of_type(&signal_type),
        }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for Print {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("trig", SignalType::Bool),
            SignalSpec::new("message", self.msg.signal_type()),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (trig, message) in iter_proc_io_as!(inputs as [bool, Any], outputs as []) {
            if let Some(message) = message {
                if message.signal_type() != self.msg.signal_type() {
                    return Err(ProcessorError::InputSpecMismatch {
                        index: 1,
                        expected: self.msg.signal_type(),
                        actual: message.signal_type(),
                    });
                }
                self.msg = message;
            }

            if trig.unwrap_or(false) {
                println!("{:?}", self.msg);
            }
        }

        Ok(())
    }
}

/// A processor that continuously outputs the current sample rate.
///
/// # Inputs
///
/// None.
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `sample_rate` | `Float` | The sample rate. |
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SampleRate;

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for SampleRate {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("sample_rate", SignalType::Float)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        outputs
            .output(0)
            .fill_as::<Float>(Some(inputs.sample_rate()));

        Ok(())
    }
}

/// A processor that smooths a signal to a target value using a smoothing factor.
///
/// The output signal will converge to the target value with a speed determined by the smoothing factor.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `target` | `Float` | The target value to smooth to. |
/// | `1` | `factor` | `Float` | The smoothing factor. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Float` | The smoothed output signal. |
#[derive(Clone, Debug, Default, Processor)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", processor_typetag)]
pub struct Smooth {
    #[input]
    target: Float,
    #[input]
    factor: Float,

    #[output]
    out: Float,
}

impl Smooth {
    /// Create a new `Smooth` processor with the given target value and smoothing factor.
    pub fn new(target: Float, factor: Float) -> Self {
        Self {
            target,
            factor,
            out: target,
        }
    }

    fn update(&mut self, _env: &ProcEnv) {
        self.factor = self.factor.clamp(0.0, 1.0);
        self.out = lerp(self.out, self.target, self.factor);
    }
}

/// A processor that outputs a signal when the input signal changes by more than a threshold.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `Float` | The input signal. |
/// | `1` | `threshold` | `Float` | The threshold for the change detection. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Bool` | The change signal. |
#[derive(Clone, Debug, Default, Processor)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", processor_typetag)]
pub struct Changed {
    last: Option<Float>,
    #[input]
    input: Float,
    #[input]
    threshold: Float,
    include_none: bool,

    #[output]
    out: bool,
}

impl Changed {
    /// Create a new `Changed` processor with the given threshold.
    pub fn new(threshold: Float, include_none: bool) -> Self {
        Self {
            last: None,
            threshold,
            include_none,
            input: 0.0,
            out: false,
        }
    }

    fn update(&mut self, _env: &ProcEnv) {
        if let Some(last) = self.last {
            self.out = (last - self.input).abs() > self.threshold;
        } else {
            self.out = self.include_none;
        }

        self.last = Some(self.input);
    }
}

/// A processor that outputs a signal when the input signal crosses zero.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `Float` | The input signal. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Bool` | The zero crossing signal. |
#[derive(Clone, Debug, Default, Processor)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", processor_typetag)]
pub struct ZeroCrossing {
    last: Float,

    #[input]
    input: Float,
    #[output]
    out: bool,
}

impl ZeroCrossing {
    /// Create a new `ZeroCrossing` processor.
    pub fn new() -> Self {
        Self {
            last: 0.0,
            input: 0.0,
            out: false,
        }
    }

    fn update(&mut self, _env: &ProcEnv) {
        self.out = (self.last < 0.0 && self.input >= 0.0) || (self.last > 0.0 && self.input <= 0.0);

        self.last = self.input;
    }
}

/// A processor that transmits a signal to a corresponding [`SignalRx`] receiver.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `Any` | The input signal. |
///
/// # Outputs
///
/// None.
#[derive(Clone, Debug)]
pub struct SignalTx {
    tx: Sender<AnySignal>,
}

impl SignalTx {
    pub(crate) fn new(tx: Sender<AnySignal>) -> Self {
        Self { tx }
    }

    /// Sends a message to the receiver.
    pub fn send(&self, message: AnySignal) {
        self.tx.try_send(message).ok();
    }
}

/// A processor that receives a signal from a corresponding [`SignalTx`] transmitter.
///
/// # Inputs
///
/// None.
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Any` | The output signal. |
#[derive(Clone, Debug)]
pub struct SignalRx {
    rx: Receiver<AnySignal>,
}

impl SignalRx {
    pub(crate) fn new(rx: Receiver<AnySignal>) -> Self {
        Self { rx }
    }

    /// Receives a message from the transmitter.
    pub fn recv(&self) -> Option<AnySignal> {
        self.rx.try_recv().ok()
    }
}

/// A wrapper around a [`SignalRx`] receiver that stores the last received message. Used as part of a [`Param`] processor.
#[derive(Clone, Debug)]
pub struct ParamRx {
    rx: SignalRx,
    last: Arc<Mutex<Option<AnySignal>>>,
}

impl ParamRx {
    pub(crate) fn new(rx: SignalRx) -> Self {
        Self {
            rx,
            last: Arc::new(Mutex::new(None)),
        }
    }

    /// Receives a message from the transmitter and stores it as the last message.
    pub fn recv(&self) -> Option<AnySignal> {
        let mut last = self.last.try_lock().ok()?;
        if let Some(msg) = self.rx.recv() {
            if let Some(last) = &mut *last {
                last.clone_from(&msg);
            } else {
                *last = Some(msg);
            }
            Some(msg)
        } else {
            None
        }
    }

    /// Returns the last received message.
    pub fn last(&self) -> Option<AnySignal> {
        *self.last.try_lock().ok()?
    }
}

/// Creates a new set of connected [`SignalTx`] and [`SignalRx`] transmitters and receivers.
pub fn signal_channel() -> (SignalTx, SignalRx) {
    let (tx, rx) = crossbeam_channel::unbounded();
    (SignalTx::new(tx), SignalRx::new(rx))
}

pub(crate) fn param_channel() -> (SignalTx, ParamRx) {
    let (tx, rx) = crossbeam_channel::unbounded();
    (SignalTx::new(tx), ParamRx::new(SignalRx::new(rx)))
}

#[derive(Clone, Debug)]
struct ParamChannel(SignalTx, ParamRx);

impl Default for ParamChannel {
    fn default() -> Self {
        let (tx, rx) = param_channel();
        Self(tx, rx)
    }
}

#[derive(Debug)]
pub(crate) struct ParamInner {
    name: String,
    channel: ParamChannel,
    signal_type: SignalType,
    minimum: Option<Float>,
    maximum: Option<Float>,
}

/// A processor that can be used to control a parameter from outside the graph.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `set` | `Any` | The value to set the parameter to. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `get` | `Any` | The current value of the parameter. |
#[derive(Clone, Debug)]
pub struct Param {
    inner: Arc<ParamInner>,
}

impl Param {
    /// Creates a new `Param` processor with the given name and optional initial value.
    pub fn new<S: Signal>(name: impl Into<String>, initial_value: impl Into<Option<S>>) -> Self {
        let this = Self {
            inner: Arc::new(ParamInner {
                name: name.into(),
                channel: ParamChannel::default(),
                signal_type: S::signal_type(),
                minimum: None,
                maximum: None,
            }),
        };
        if let Some(initial_value) = initial_value.into() {
            this.send(initial_value);
        }
        this
    }

    /// Creates a new `Param` processor with the given name and optional initial value, minimum, and maximum.
    pub fn bounded(
        name: impl Into<String>,
        initial_value: impl Into<Option<Float>>,
        minimum: impl Into<Option<Float>>,
        maximum: impl Into<Option<Float>>,
    ) -> Self {
        let this = Self {
            inner: Arc::new(ParamInner {
                name: name.into(),
                channel: ParamChannel::default(),
                signal_type: SignalType::Float,
                minimum: minimum.into(),
                maximum: maximum.into(),
            }),
        };
        if let Some(initial_value) = initial_value.into() {
            this.send(initial_value);
        }
        this
    }

    /// Returns the name of the parameter.
    pub fn name(&self) -> &str {
        &self.inner.name
    }

    /// Returns the signal type of the parameter.
    pub fn signal_type(&self) -> SignalType {
        self.inner.signal_type
    }

    /// Returns the transmitter for the parameter.
    pub fn tx(&self) -> &SignalTx {
        &self.inner.channel.0
    }

    /// Returns the receiver for the parameter.
    pub fn rx(&self) -> &ParamRx {
        &self.inner.channel.1
    }

    /// Sends a value to the parameter.
    pub fn send(&self, message: impl Signal) {
        let message = message.into_any_signal();
        match (message, self.inner.minimum, self.inner.maximum) {
            (AnySignal::Float(value), Some(min), Some(max)) => {
                self.tx().send(AnySignal::Float(value.clamp(min, max)));
            }
            (AnySignal::Float(value), Some(min), None) => {
                self.tx().send(AnySignal::Float(value.max(min)));
            }
            (AnySignal::Float(value), None, Some(max)) => {
                self.tx().send(AnySignal::Float(value.min(max)));
            }
            (message, _, _) => self.tx().send(message),
        }
    }

    /// Receives the value of the parameter.
    pub fn recv(&self) -> Option<AnySignal> {
        let message = self.rx().recv();

        match (message, self.inner.minimum, self.inner.maximum) {
            (Some(AnySignal::Float(value)), Some(min), Some(max)) => {
                Some(AnySignal::Float(value.clamp(min, max)))
            }
            (Some(AnySignal::Float(value)), Some(min), None) => {
                Some(AnySignal::Float(value.max(min)))
            }
            (Some(AnySignal::Float(value)), None, Some(max)) => {
                Some(AnySignal::Float(value.min(max)))
            }
            (message, _, _) => message,
        }
    }

    /// Returns the last received value of the parameter.
    pub fn last(&self) -> Option<AnySignal> {
        let last = self.rx().last();

        match (last, self.inner.minimum, self.inner.maximum) {
            (Some(AnySignal::Float(value)), Some(min), Some(max)) => {
                Some(AnySignal::Float(value.clamp(min, max)))
            }
            (Some(AnySignal::Float(value)), Some(min), None) => {
                Some(AnySignal::Float(value.max(min)))
            }
            (Some(AnySignal::Float(value)), None, Some(max)) => {
                Some(AnySignal::Float(value.min(max)))
            }
            (last, _, _) => last,
        }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for Param {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("set", self.signal_type())]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("get", self.signal_type())]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (set, mut get) in iter_proc_io_as!(inputs as [Any], outputs as [Any]) {
            if let Some(Some(set)) = set.map(|signal| signal.try_into_any_signal()) {
                self.tx().send(set.to_owned());
            }

            if let Some(msg) = self.rx().recv() {
                get.set_any(msg);
            } else if let Some(last) = self.rx().last() {
                get.set_any(last);
            } else {
                get.set_none();
            }
        }

        Ok(())
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Param {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(serde::Serialize)]
        struct ParamSer {
            name: String,
            signal_type: SignalType,
            minimum: Option<Float>,
            maximum: Option<Float>,
            initial_value: Option<AnySignal>,
        }

        self.recv();

        let ser = ParamSer {
            name: self.name.clone(),
            signal_type: self.signal_type,
            minimum: self.minimum,
            maximum: self.maximum,
            initial_value: self.last(),
        };

        ser.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Param {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct ParamDe {
            name: String,
            signal_type: SignalType,
            minimum: Option<Float>,
            maximum: Option<Float>,
            initial_value: Option<AnySignal>,
        }

        let de = ParamDe::deserialize(deserializer)?;

        let param = Param {
            name: de.name,
            channel: ParamChannel::default(),
            signal_type: de.signal_type,
            minimum: de.minimum,
            maximum: de.maximum,
        };
        if let Some(initial_value) = de.initial_value {
            param.tx().send(initial_value);
        }

        Ok(param)
    }
}

/// A processor that counts the number of times it has been triggered.
///
/// The counter is reset to zero when the reset signal is `true`.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `trig` | `Bool` | The trigger signal. |
/// | `1` | `reset` | `Bool` | The reset signal. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `count` | `Int` | The current count. |
#[derive(Clone, Debug, Default, Processor)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", processor_typetag)]
pub struct Counter {
    #[input]
    trig: bool,
    #[input]
    reset: bool,

    #[output]
    count: i64,
}

impl Counter {
    /// Create a new `Counter` processor.
    pub fn new() -> Self {
        Self {
            trig: false,
            reset: false,
            count: 0,
        }
    }

    fn update(&mut self, _env: &ProcEnv) {
        if self.reset {
            self.count = 0;
        }

        if self.trig {
            self.count += 1;
        }
    }
}

/// A processor that captures the value of a signal when triggered and contuously outputs it.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `Float` | The input signal. |
/// | `1` | `trig` | `Bool` | The trigger signal. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Float` | The output signal. |
#[derive(Clone, Debug, Default, Processor)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", processor_typetag)]
pub struct SampleAndHold {
    last: Option<Float>,

    #[input]
    input: Float,
    #[input]
    trig: bool,

    #[output]
    out: Float,
}

impl SampleAndHold {
    /// Create a new `SampleAndHold` processor.
    pub fn new() -> Self {
        Self {
            last: None,
            input: 0.0,
            trig: false,
            out: 0.0,
        }
    }

    fn update(&mut self, _env: &ProcEnv) {
        if self.trig {
            self.last = Some(self.input);
        }

        self.out = self.last.unwrap_or(self.input);
    }
}

/// A processor that panics with a message if the input signal is NaN or infinite.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `Float` | The input signal. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Float` | The input signal passed through. |
#[derive(Clone, Debug, Default, Processor)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", processor_typetag)]
pub struct CheckFinite {
    context: String,

    #[input]
    input: Float,
    #[output]
    out: Float,
}

impl CheckFinite {
    /// Create a new `CheckFinite` processor with the given context for the panic message.
    pub fn new(context: impl Into<String>) -> Self {
        Self {
            context: context.into(),
            input: 0.0,
            out: 0.0,
        }
    }

    fn update(&mut self, _env: &ProcEnv) {
        if self.input.is_nan() || self.input.is_infinite() {
            panic!("{}: input signal is NaN or infinite", self.context);
        }

        self.out = self.input;
    }
}

/// A processor that outputs 0.0 when the input signal is NaN or infinite.
/// Otherwise, it passes the input signal through unchanged.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `Float` | The input signal. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Float` | The input signal passed through, or 0.0 if the input signal is NaN or infinite. |
#[derive(Clone, Debug, Default, Processor)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", processor_typetag)]
pub struct FiniteOrZero {
    #[input]
    input: Float,
    #[output]
    out: Float,
}

impl FiniteOrZero {
    /// Create a new `FiniteOrZero` processor.
    pub fn new() -> Self {
        Self {
            input: 0.0,
            out: 0.0,
        }
    }

    fn update(&mut self, _env: &ProcEnv) {
        if self.input.is_nan() || self.input.is_infinite() {
            self.out = 0.0;
        } else {
            self.out = self.input;
        }
    }
}

/// A processor that deduplicates a signal by only outputting a new value when it changes.
///
/// This can be thought of as the opposite of the [`Register`] processor, and will effectively undo its effect.
///
/// The output signal will likely be much sparser than the input signal, reducing the amount of data that needs to be processed downstream.
///
/// This processor can be useful when placed before an expensive processor (such as those dealing with lists) to reduce the amount of work it needs to do.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `Any` | The input signal. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Any` | The deduplicated output signal. |
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Dedup {
    signal_type: SignalType,
    last: Option<AnySignal>,
}

impl Dedup {
    /// Create a new `Dedup` processor.
    pub fn new(signal_type: SignalType) -> Self {
        Self {
            signal_type,
            last: None,
        }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for Dedup {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("in", self.signal_type)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", self.signal_type)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (in_signal, mut out_signal) in iter_proc_io_as!(inputs as [Any], outputs as [Any]) {
            if let Some(Some(in_signal)) =
                in_signal.map(|in_signal| in_signal.try_into_any_signal())
            {
                if in_signal != self.last.unwrap_or_else(|| in_signal.to_owned()) {
                    out_signal.set_any(in_signal.to_owned());
                    self.last = Some(in_signal.to_owned());
                }
            }
        }

        Ok(())
    }
}

/// A processor that outputs `true` when the input signal is `Some`, and `false` when it is `None`.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `Any` | The input signal. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Bool` | The output signal. |
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IsSome {
    signal_type: SignalType,
}

impl IsSome {
    /// Create a new `IsSome` processor.
    pub fn new(signal_type: SignalType) -> Self {
        Self { signal_type }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for IsSome {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("in", self.signal_type)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalType::Bool)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (in_signal, mut out_signal) in iter_proc_io_as!(inputs as [Any], outputs as [Any]) {
            if let Some(Some(_)) = in_signal
                .map(|in_signal| in_signal.try_into_any_signal())
                .as_ref()
            {
                out_signal.set(true);
            } else {
                out_signal.set(false);
            }
        }

        Ok(())
    }
}

/// A processor that outputs `true` when the input signal is `None`, and `false` when it is `Some`.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `Any` | The input signal. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Bool` | The output signal. |
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IsNone {
    signal_type: SignalType,
}

impl IsNone {
    /// Create a new `IsNone` processor.
    pub fn new(signal_type: SignalType) -> Self {
        Self { signal_type }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for IsNone {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("in", self.signal_type)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalType::Bool)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (in_signal, mut out_signal) in iter_proc_io_as!(inputs as [Any], outputs as [Any]) {
            if let Some(Some(_)) = in_signal
                .map(|in_signal| in_signal.try_into_any_signal())
                .as_ref()
            {
                out_signal.set(false);
            } else {
                out_signal.set(true);
            }
        }

        Ok(())
    }
}

/// A processor that outputs the input signal if it is Some, otherwise it outputs a default value.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `in` | `Any` | The input signal. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Any` | The input signal if it is Some, otherwise a default value. |
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OrElse {
    default: AnySignal,
}

impl OrElse {
    /// Create a new `OrElse` processor.
    pub fn new(default: impl Signal) -> Self {
        Self {
            default: default.into_any_signal(),
        }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for OrElse {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("in", self.default.signal_type())]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", self.default.signal_type())]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (in_signal, mut out_signal) in iter_proc_io_as!(inputs as [Any], outputs as [Any]) {
            if let Some(Some(in_signal)) = in_signal
                .map(|in_signal| in_signal.try_into_any_signal())
                .as_ref()
            {
                out_signal.set_any(in_signal.to_owned());
            } else {
                out_signal.set_any(self.default.to_owned());
            }
        }

        Ok(())
    }
}
