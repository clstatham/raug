#![doc = include_str!("../README.md")]
#![cfg_attr(doc, warn(missing_docs))]

pub mod builtins;
pub mod graph;
pub mod processor;
#[macro_use]
pub mod signal;
pub mod util;

extern crate self as raug;

/// Re-exports of commonly used types and traits from the crate.
pub mod prelude {
    pub use crate::builtins::*;
    pub use crate::graph::{
        Graph,
        node::{Input, IntoNode, IntoOutput, Node, Output},
        runtime::{AudioBackend, AudioDevice, AudioStream, CpalStream, WavFileOutStream},
        sub_graph::SubGraph,
    };
    pub use crate::processor::{
        ProcResult, Processor, ProcessorError,
        io::{ProcEnv, ProcessorInputs, ProcessorOutputs, SignalSpec},
    };
    pub use crate::signal::{
        List, Signal, SignalType, StringSignal,
        buffer::Buffer,
        type_erased::{AnyBuffer, AnySignalMut, AnySignalRef},
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
