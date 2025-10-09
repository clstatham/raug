pub mod graph;
pub mod node;
#[cfg(feature = "osc")]
pub mod osc;
pub mod processors;

pub mod prelude {
    pub use crate::graph::GraphExt;
    #[cfg(feature = "osc")]
    pub use crate::osc::{OscClient, OscType};
    pub use crate::processors::*;
}

#[doc(hidden)]
pub use raug as __raug;
