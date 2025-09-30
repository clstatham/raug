use std::{
    any::TypeId,
    fmt::{Debug, Display},
    hash::{Hash, Hasher},
};

use thiserror::Error;

pub use petgraph;

use graph::NodeIndex;

pub mod builder;
pub mod graph;
pub mod node;

pub mod prelude {
    pub use crate::{
        builder::{GraphBuilder, NodeBuilder},
        graph::{AbstractGraph, Connection, DuplicateConnectionMode, Graph, NodeIndex},
        node::AbstractNode,
    };
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Input index {index} out of bounds for node {node:?} (only has {num_inputs} inputs)")]
    InputIndexOutOfBounds {
        node: NodeIndex,
        index: String,
        num_inputs: usize,
    },

    #[error(
        "Output index {index} out of bounds for node {node:?} (only has {num_outputs} outputs)"
    )]
    OutputIndexOutOfBounds {
        node: NodeIndex,
        index: String,
        num_outputs: usize,
    },

    #[error(
        "Duplicate connection on input {target_input} of node {target:?}: Already connected to output {src_output} of node {src:?}"
    )]
    DuplicateConnection {
        src: NodeIndex,
        src_output: u32,
        target: NodeIndex,
        target_input: u32,
    },

    #[error("Type mismatch: expected {expected}, got {got}")]
    TypeMismatch { expected: TypeInfo, got: TypeInfo },

    #[error("Node mismatch: expected node {expected:?}, got node {got:?}")]
    NodeMismatch { expected: NodeIndex, got: NodeIndex },
}

#[derive(Clone, Copy)]
pub struct TypeInfo {
    pub type_name: &'static str,
    pub type_id: TypeId,
}

impl Debug for TypeInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TypeInfo({})", self.type_name)
    }
}

impl Display for TypeInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.type_name)
    }
}

impl PartialEq for TypeInfo {
    fn eq(&self, other: &Self) -> bool {
        self.type_id == other.type_id
    }
}

impl Eq for TypeInfo {}

impl Hash for TypeInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.type_id.hash(state);
    }
}

impl TypeInfo {
    pub fn of<T: ?Sized + 'static>() -> Self {
        Self {
            type_name: std::any::type_name::<T>(),
            type_id: TypeId::of::<T>(),
        }
    }
}

pub trait GetTypeInfo: 'static {
    fn type_info() -> TypeInfo {
        TypeInfo::of::<Self>()
    }
}

impl<T: ?Sized + 'static> GetTypeInfo for T {}
