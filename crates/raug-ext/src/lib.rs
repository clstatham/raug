pub mod graph;
pub mod node;
pub mod processors;

pub mod prelude {
    pub use crate::graph::GraphExt;
    pub use crate::node::OutputExt;
    pub use crate::processors::*;
}
