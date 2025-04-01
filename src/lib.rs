#![doc = include_str!("../README.md")]
#![cfg_attr(doc, warn(missing_docs))]
#![allow(clippy::useless_conversion)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::excessive_precision)]

pub mod builder;
pub mod builtins;
pub mod graph;
pub mod midi;
pub mod processor;
pub mod runtime;
pub mod signal;
pub mod util;

#[cfg(feature = "fft")]
pub mod fft;

#[cfg(feature = "fft")]
pub use fft::builtins as fft_builtins;

extern crate self as raug;

/// Re-exports of commonly used types and traits from the crate.
#[allow(unused_imports)]
pub mod prelude {
    pub use crate::builder::{
        graph_builder::GraphBuilder,
        node_builder::{Input, IntoNode, Node, Output},
    };
    pub use crate::builtins::*;
    pub use crate::graph::Graph;
    pub use crate::midi::MidiMessage;
    pub use crate::processor::{
        ProcEnv, Processor, ProcessorError, ProcessorInputs, ProcessorOutputs, SignalSpec,
    };
    pub use crate::runtime::{AudioBackend, AudioDevice, MidiPort, Runtime, RuntimeHandle};
    pub use crate::signal::{
        AnySignal, AnySignalOpt, Float, OptRepr, OptSignal, PI, Signal, SignalBuffer, SignalType,
        TAU, buffer::Buffer, optional::Repr,
    };
    pub use crate::util::*;
    pub use raug_macros::{Processor, iter_proc_io_as, note, note_array};
    pub use std::time::Duration;

    #[cfg(feature = "fft")]
    pub use crate::fft::{
        WindowFunction,
        builder::{FftGraphBuilder, FftNode},
        graph::FftGraph,
        processor::{FftProcessor, FftSpec},
        signal::{ComplexBuf, FftBufLength, FftSignal, FftSignalType, RealBuf},
    };
}

#[doc(hidden)]
mod graph_serde {
    #[cfg(feature = "serde")]
    pub trait GraphSerde: erased_serde::Serialize {}
    #[cfg(feature = "serde")]
    impl<T: ?Sized> GraphSerde for T where T: erased_serde::Serialize {}

    #[cfg(not(feature = "serde"))]
    pub trait GraphSerde {}
    #[cfg(not(feature = "serde"))]
    impl<T: ?Sized> GraphSerde for T {}
}

pub(crate) use graph_serde::GraphSerde;

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

#[doc(hidden)]
#[allow(unused)]
#[cfg(feature = "serde")]
pub mod __typetag {
    pub use typetag::*;
}
