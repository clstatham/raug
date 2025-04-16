use std::sync::{Arc, Mutex, RwLock};

use crate::{graph::runtime::Channels, prelude::*};

/// A processor that can be used to control a signal value remotely.
#[derive(Clone)]
pub struct Param<T: Signal + Clone> {
    name: Arc<Option<String>>,
    channels: Arc<Channels<T>>,
    last: Arc<RwLock<T>>,
}

impl<T: Signal + Clone> Param<T> {
    pub fn new(name: impl Into<Option<String>>, initial_value: T) -> Self {
        Self {
            name: Arc::new(name.into()),
            channels: Arc::new(Channels::unbounded()),
            last: Arc::new(RwLock::new(initial_value)),
        }
    }

    pub fn set(&self, value: T) {
        self.channels.try_send(value).unwrap();
    }

    pub fn get(&self) -> T {
        if let Some(new) = self.channels.try_recv().unwrap() {
            *self.last.write().unwrap() = new.clone();
            new
        } else {
            self.last.read().unwrap().clone()
        }
    }
}

impl<T: Signal + Default + Clone> Default for Param<T> {
    fn default() -> Self {
        Self::new(None, T::default())
    }
}

impl<T: Signal + Clone> Processor for Param<T> {
    fn name(&self) -> &str {
        match &*self.name {
            Some(name) => name,
            None => "Param",
        }
    }

    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", <Option<T>>::signal_type())]
    }

    fn create_output_buffers(&self, size: usize) -> Vec<AnyBuffer> {
        vec![AnyBuffer::zeros::<Option<T>>(size)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for sample_index in 0..inputs.block_size() {
            let value = self.get();
            outputs.set_output_as(0, sample_index, &value)?;
        }

        Ok(())
    }
}
