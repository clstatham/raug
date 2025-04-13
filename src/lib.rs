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
#[allow(unused_imports)]
pub mod prelude {
    pub use crate::builtins::*;
    pub use crate::graph::{
        Graph,
        node::{Input, IntoNode, Node, Output},
        runtime::{AudioBackend, AudioDevice, AudioStream, CpalStream, MidiPort, WavFileOutStream},
    };
    pub use crate::processor::{
        ProcResult, Processor, ProcessorError,
        io::{ProcEnv, ProcessorInputs, ProcessorOutputs, SignalSpec},
    };
    pub use crate::signal::{
        OptRepr, OptSignal, Signal, SignalType,
        buffer::Buffer,
        repr::Repr,
        type_erased::{ErasedBuffer, ErasedSignalMut, ErasedSignalRef},
    };
    pub use crate::util::*;
    pub use raug_macros::{note, note_array, processor};
    pub use std::time::Duration;
}

#[doc(hidden)]
#[allow(unused)]
mod logging {
    use std::{
        collections::HashSet,
        sync::{LazyLock, Mutex},
    };

    pub(crate) static LOGGED: LazyLock<Mutex<HashSet<String>>> =
        LazyLock::new(|| Mutex::new(HashSet::with_capacity(16)));

    #[macro_export]
    macro_rules! log_once {
        ($val:expr => error $($msg:tt)*) => {{
            if log::max_level() >= log::LevelFilter::Error && $crate::logging::LOGGED.lock().unwrap().insert($val.to_string()) {
                log::error!($($msg)*);
            }
        }};
        ($val:expr => warn $($msg:tt)*) => {{
            if log::max_level() >= log::LevelFilter::Warn && $crate::logging::LOGGED.lock().unwrap().insert($val.to_string()) {
                log::warn!($($msg)*);
            }
        }};
        ($val:expr => info $($msg:tt)*) => {{
            if log::max_level() >= log::LevelFilter::Info && $crate::logging::LOGGED.lock().unwrap().insert($val.to_string()) {
                log::info!($($msg)*);
            }
        }};
        ($val:expr => debug $($msg:tt)*) => {{
            if log::max_level() >= log::LevelFilter::Debug && $crate::logging::LOGGED.lock().unwrap().insert($val.to_string()) {
                log::debug!($($msg)*);
            }
        }};
        ($val:expr => trace $($msg:tt)*) => {{
            if log::max_level() >= log::LevelFilter::Trace && $crate::logging::LOGGED.lock().unwrap().insert($val.to_string()) {
                log::trace!($($msg)*);
            }
        }};
    }

    #[macro_export]
    macro_rules! error_once {
        ($val:expr => $($msg:tt)*) => {
            $crate::log_once!($val => error $($msg)*);
        };

        ($($val:tt)*) => {
            $crate::log_once!(format!($($val)*) => error $($val)*);
        };
    }

    #[macro_export]
    macro_rules! warn_once {
        ($val:expr => $($msg:tt)*) => {
            $crate::log_once!($val => warn $($msg)*);
        };

        ($($val:tt)*) => {
            $crate::log_once!(format!($($val)*) => warn $($val)*);
        };
    }

    #[macro_export]
    macro_rules! info_once {
        ($val:expr => $($msg:tt)*) => {
            $crate::log_once!($val => info $($msg)*);
        };

        ($($val:tt)*) => {
            $crate::log_once!(format!($($val)*) => info $($val)*);
        };
    }

    #[macro_export]
    macro_rules! debug_once {
        ($val:expr => $($msg:tt)*) => {
            $crate::log_once!($val => debug $($msg)*);
        };

        ($($val:tt)*) => {
            $crate::log_once!(format!($($val)*) => debug $($val)*);
        };
    }

    #[macro_export]
    macro_rules! trace_once {
        ($val:expr => $($msg:tt)*) => {
            $crate::log_once!($val => trace $($msg)*);
        };

        ($($val:tt)*) => {
            $crate::log_once!(format!($($val)*) => trace $($val)*);
        };
    }
}

#[doc(hidden)]
#[allow(unused)]
pub mod __itertools {
    pub use itertools::*;
}
