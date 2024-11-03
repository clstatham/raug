use crossbeam_channel::{Receiver, Sender};
use daprs::prelude::*;

pub struct GuiChannel {
    tx: GuiTx,
    rx: GuiRx,
}

impl GuiChannel {
    pub fn new() -> Self {
        let (tx, rx) = crossbeam_channel::unbounded();
        Self {
            tx: GuiTx::new(tx),
            rx: GuiRx::new(rx),
        }
    }

    pub fn tx(&self) -> &GuiTx {
        &self.tx
    }

    pub fn rx(&self) -> &GuiRx {
        &self.rx
    }
}

impl Default for GuiChannel {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct GuiTx {
    tx: Sender<BoxedMessage>,
}

impl GuiTx {
    pub fn new(tx: Sender<BoxedMessage>) -> Self {
        Self { tx }
    }

    pub fn send(&self, message: impl Message) {
        self.tx.try_send(Box::new(message)).ok();
    }
}

#[derive(Debug, Clone)]
pub struct GuiRx {
    rx: Receiver<BoxedMessage>,
    last_message: Option<BoxedMessage>,
}

impl GuiRx {
    pub fn new(rx: Receiver<BoxedMessage>) -> Self {
        Self {
            rx,
            last_message: None,
        }
    }

    pub fn recv(&mut self) -> Option<&BoxedMessage> {
        if let Ok(msg) = self.rx.try_recv() {
            self.last_message = Some(msg.clone());
        }
        self.last_message.as_ref()
    }

    pub fn last_message(&self) -> Option<&BoxedMessage> {
        self.last_message.as_ref()
    }
}

impl Process for GuiRx {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("out", Signal::new_message_none())]
    }

    fn process(
        &mut self,
        _inputs: &[SignalBuffer],
        outputs: &mut [SignalBuffer],
    ) -> Result<(), ProcessorError> {
        let out = outputs[0]
            .as_message_mut()
            .ok_or(ProcessorError::OutputSpecMismatch(0))?;

        for out in out {
            if let Some(msg) = self.recv() {
                *out = Some(msg.clone());
            } else {
                *out = None;
            }
        }

        Ok(())
    }
}
