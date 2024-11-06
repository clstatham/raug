//! Time-related processors.

use crate::{
    message::Message,
    prelude::{GraphBuilder, Node, Process, SignalSpec},
    processor::ProcessorError,
    signal::Signal,
};

/// A metronome that emits a bang at the given period.
///
/// # Inputs
///
/// | Index | Name | Type | Default | Description |
/// | --- | --- | --- | --- | --- |
/// | `0` | `period` | `Message(f64)` | | The period of the metronome in seconds. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Bang` | Emits a bang at the given period. |
#[derive(Debug, Clone)]

pub struct Metro {
    period: f64,
    last_time: f64,
    next_time: f64,
    time: f64,
    sample_rate: f64,
}

impl Metro {
    /// Creates a new metronome processor with the given period.
    pub fn new() -> Self {
        Self {
            period: 1.0,
            last_time: 0.0,
            next_time: 0.0,
            time: 0.0,
            sample_rate: 0.0,
        }
    }

    fn next_sample(&mut self) -> bool {
        let out = if self.time >= self.next_time {
            self.last_time = self.time;
            self.next_time = self.time + self.period;
            true
        } else {
            false
        };

        self.time += self.sample_rate.recip();

        out
    }
}

impl Default for Metro {
    fn default() -> Self {
        Self::new()
    }
}

impl Process for Metro {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("period", Signal::new_message_none())]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("out", Signal::new_message_none())]
    }

    fn resize_buffers(&mut self, sample_rate: f64, _block_size: usize) {
        self.sample_rate = sample_rate;
    }

    fn process(
        &mut self,
        inputs: &[crate::signal::SignalBuffer],
        outputs: &mut [crate::signal::SignalBuffer],
    ) -> Result<(), ProcessorError> {
        let period = inputs[0]
            .as_message()
            .ok_or(ProcessorError::InputSpecMismatch(0))?;

        let out = outputs[0]
            .as_message_mut()
            .ok_or(ProcessorError::OutputSpecMismatch(0))?;

        for (period, out) in itertools::izip!(period, out) {
            if let Some(period) = period {
                if let Some(period) = period.cast_to_float() {
                    self.period = period;
                }
            }

            if self.next_sample() {
                *out = Some(Message::Bang);
            } else {
                *out = None;
            }
        }

        Ok(())
    }
}

impl GraphBuilder {
    /// A metronome processor.
    ///
    /// See also: [`Metro`](crate::builtins::time::Metro).
    pub fn metro(&self) -> Node {
        self.add_processor(Metro::new())
    }
}
