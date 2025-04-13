#![doc = include_str!("../README.md")]
#![cfg_attr(doc, warn(missing_docs))]
#![allow(clippy::useless_conversion)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::excessive_precision)]

pub mod builtins;
pub mod graph;
pub mod processor;
pub mod signal;
pub mod util;

extern crate self as raug;

/// Re-exports of commonly used types and traits from the crate.
pub mod prelude {
    pub use crate::builtins::*;
    pub use crate::graph::{
        Graph,
        node::{Input, IntoNode, IntoOutput, Node, Output},
        runtime::{AudioBackend, AudioDevice, AudioStream, CpalStream, MidiPort, WavFileOutStream},
    };
    pub use crate::processor::{
        ProcResult, Processor, ProcessorError,
        io::{ProcEnv, ProcessorInputs, ProcessorOutputs, SignalSpec},
    };
    pub use crate::signal::{
        Signal, SignalType,
        buffer::Buffer,
        type_erased::{ErasedBuffer, ErasedSignalMut, ErasedSignalRef},
    };
    pub use crate::util::*;
    pub use raug_macros::{note, note_array, processor};
    pub use std::time::Duration;
}

#[doc(hidden)]
#[allow(unused)]
pub mod __itertools {
    pub use itertools::*;
}
