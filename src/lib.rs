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
    pub use crate::{
        builtins::*,
        graph::{
            Graph, RunningGraph,
            node::{Input, IntoNode, IntoOutputExt, IntoOutputOpt, Node, Output},
            runtime::{AudioBackend, AudioDevice, AudioOut, CpalOut, WavFileOut},
            sub_graph::SubGraph,
        },
        processor::{
            ProcResult, Processor, ProcessorError,
            io::{ProcEnv, ProcessorInputs, ProcessorOutputs, SignalSpec},
        },
        signal::{
            List, Signal, SignalType, Str,
            type_erased::{AnyBuffer, AnySignalMut, AnySignalRef},
        },
        util::*,
    };
    pub use raug_macros::{note, note_array, processor};
    pub use std::time::Duration;
}

#[doc(hidden)]
#[allow(unused)]
pub mod __itertools {
    pub use itertools::*;
}
