use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Debug,
    marker::PhantomData,
    ops::{Index, IndexMut},
};

use petgraph::prelude::*;
use rustc_hash::FxHashSet;

use crate::{Error, node::AbstractNode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DuplicateConnectionMode {
    Disconnect,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisitResult<T> {
    Continue,
    Break(T),
    Done,
}

pub trait AbstractGraph {
    type Node: AbstractNode;
    type Edge;

    fn duplicate_connection_mode() -> DuplicateConnectionMode {
        DuplicateConnectionMode::Error
    }

    fn graph(&self) -> &Graph<Self>;
    fn graph_mut(&mut self) -> &mut Graph<Self>;
}

pub type NodeIndex = petgraph::graph::NodeIndex;
pub type EdgeIndex = petgraph::graph::EdgeIndex;

#[derive(Debug, Clone, Copy)]
pub struct Connection<G: AbstractGraph + ?Sized> {
    pub source: NodeIndex,
    pub source_output: u32,
    pub target: NodeIndex,
    pub target_input: u32,
    _g: PhantomData<G>,
}

pub struct Graph<G: AbstractGraph + ?Sized> {
    pub(crate) digraph: StableDiGraph<G::Node, Connection<G>>,

    // cached input/output nodes
    inputs: Vec<NodeIndex>,
    outputs: Vec<NodeIndex>,

    // cached visitor state for graph traversal
    visitor: DfsPostOrder<NodeIndex, FxHashSet<NodeIndex>>,
    visit_path: Vec<NodeIndex>,
    pub needs_visitor_reset: bool,
}

impl<G: AbstractGraph> Default for Graph<G> {
    fn default() -> Self {
        Self {
            digraph: StableDiGraph::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            visitor: DfsPostOrder::default(),
            visit_path: Vec::new(),
            needs_visitor_reset: false,
        }
    }
}

impl<G: AbstractGraph> Graph<G> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn inputs(&self) -> &[NodeIndex] {
        &self.inputs
    }

    pub fn outputs(&self) -> &[NodeIndex] {
        &self.outputs
    }

    pub fn num_inputs(&self) -> usize {
        self.inputs.len()
    }

    pub fn num_outputs(&self) -> usize {
        self.outputs.len()
    }

    pub fn digraph(&self) -> &StableDiGraph<G::Node, Connection<G>> {
        &self.digraph
    }

    pub fn digraph_mut(&mut self) -> &mut StableDiGraph<G::Node, Connection<G>> {
        &mut self.digraph
    }

    pub fn visit_path(&self) -> &[NodeIndex] {
        &self.visit_path
    }

    pub fn add_input(&mut self, node: G::Node) -> NodeIndex {
        let idx = self.digraph.add_node(node);
        self.inputs.push(idx);
        idx
    }

    pub fn add_output(&mut self, node: G::Node) -> NodeIndex {
        let idx = self.digraph.add_node(node);
        self.outputs.push(idx);
        idx
    }

    pub fn add_node(&mut self, node: G::Node) -> NodeIndex {
        self.digraph.add_node(node)
    }

    pub fn connect(
        &mut self,
        source: NodeIndex,
        source_output: u32,
        target: NodeIndex,
        target_input: u32,
    ) -> Result<EdgeIndex, Error> {
        let dupe_mode = G::duplicate_connection_mode();
        if let Some(dupe) = self
            .digraph
            .edges_directed(target, Direction::Incoming)
            .find(|edge| edge.weight().target_input == target_input)
        {
            match dupe_mode {
                DuplicateConnectionMode::Disconnect => {
                    // disconnect the existing edge
                    self.digraph.remove_edge(dupe.id());
                }
                DuplicateConnectionMode::Error => {
                    // error, we cannot have duplicate connections
                    return Err(Error::DuplicateConnection {
                        src: dupe.weight().source,
                        src_output: dupe.weight().source_output,
                        target,
                        target_input,
                    });
                }
            }
        }

        // type check
        let source_node = &self.digraph[source];
        let target_node = &self.digraph[target];
        let Some(source_type) = source_node.output_type(source_output) else {
            return Err(Error::InputIndexOutOfBounds {
                node: source,
                index: source_output.to_string(),
                num_inputs: source_node.num_inputs(),
            });
        };
        let Some(target_type) = target_node.input_type(target_input) else {
            return Err(Error::OutputIndexOutOfBounds {
                node: target,
                index: target_input.to_string(),
                num_outputs: target_node.num_outputs(),
            });
        };
        if source_type != target_type {
            return Err(Error::TypeMismatch {
                expected: target_type,
                got: source_type,
            });
        }

        let connection = Connection {
            source,
            source_output,
            target,
            target_input,
            _g: PhantomData,
        };

        self.needs_visitor_reset = true;

        Ok(self.digraph.add_edge(source, target, connection))
    }

    pub fn disconnect(&mut self, target: NodeIndex, target_input: u32) -> Option<Connection<G>> {
        if let Some(edge) = self
            .digraph
            .edges_directed(target, Direction::Incoming)
            .find(|edge| edge.weight().target_input == target_input)
        {
            let connection = self.digraph.remove_edge(edge.id()).unwrap();
            self.needs_visitor_reset = true;
            Some(connection)
        } else {
            None
        }
    }

    pub fn disconnect_all_inputs(&mut self, node: NodeIndex) -> Vec<Connection<G>> {
        let mut disconnected = Vec::new();
        let inputs: Vec<_> = self
            .digraph
            .edges_directed(node, Direction::Incoming)
            .map(|e| e.id())
            .collect();

        for edge in inputs {
            let connection = self.digraph.remove_edge(edge).unwrap();
            disconnected.push(connection);
        }

        self.needs_visitor_reset = true;

        disconnected
    }

    pub fn disconnect_all_outputs(&mut self, node: NodeIndex) -> Vec<Connection<G>> {
        let mut disconnected = Vec::new();
        let outputs: Vec<_> = self
            .digraph
            .edges_directed(node, Direction::Outgoing)
            .map(|e| e.id())
            .collect();

        for edge in outputs {
            let connection = self.digraph.remove_edge(edge).unwrap();
            disconnected.push(connection);
        }

        self.needs_visitor_reset = true;

        disconnected
    }

    pub fn disconnect_all(&mut self, node: NodeIndex) -> Vec<Connection<G>> {
        let mut disconnected = Vec::new();
        disconnected.extend(self.disconnect_all_inputs(node));
        disconnected.extend(self.disconnect_all_outputs(node));
        self.needs_visitor_reset = true;
        disconnected
    }

    pub fn garbage_collect(&mut self) -> Vec<G::Node> {
        let all_nodes: BTreeSet<NodeIndex> = self.digraph().node_indices().collect();
        let mut removed_nodes = BTreeMap::default();
        let mut again = false;

        while !again {
            again = false;
            for node in all_nodes.iter() {
                if removed_nodes.contains_key(node) {
                    continue;
                }

                let is_output = self.outputs.contains(node);
                let has_children = self
                    .digraph
                    .edges_directed(*node, Direction::Outgoing)
                    .count()
                    > 0;

                if !is_output && !has_children {
                    let weight = self.digraph.remove_node(*node).unwrap();
                    removed_nodes.insert(*node, weight);
                    self.needs_visitor_reset = true;
                    again = true;
                }
            }
        }

        removed_nodes.into_values().collect()
    }

    pub fn reset_visitor(&mut self) {
        if !self.needs_visitor_reset {
            return;
        }

        if self.visit_path.capacity() < self.digraph.node_count() {
            let additional = self.digraph.node_count() - self.visit_path.capacity();
            self.visit_path.reserve(additional);
        }

        self.visit_path.clear();
        self.visitor.discovered.clear();
        self.visitor.stack.clear();
        self.visitor.finished.clear();

        for node in self.digraph.externals(Direction::Incoming) {
            self.visitor.stack.push(node);
        }

        while let Some(node) = self.visitor.next(&self.digraph) {
            self.visit_path.push(node);
        }

        self.visit_path.reverse();
        self.needs_visitor_reset = false;
    }

    pub fn visit_mut<F, T>(&mut self, mut visit_node: F) -> VisitResult<T>
    where
        F: FnMut(NodeIndex, &mut G::Node) -> VisitResult<T>,
    {
        self.reset_visitor();

        for &node in &self.visit_path {
            let weight = &mut self.digraph[node];
            let res = visit_node(node, weight);
            if !matches!(res, VisitResult::Continue) {
                return res;
            }
        }

        VisitResult::Done
    }

    pub fn try_visit_mut<F, T, E>(&mut self, mut visit_node: F) -> Result<VisitResult<T>, E>
    where
        F: FnMut(NodeIndex, &mut G::Node) -> Result<VisitResult<T>, E>,
    {
        self.reset_visitor();

        for &node in &self.visit_path {
            let weight = &mut self.digraph[node];
            let res = visit_node(node, weight)?;
            if !matches!(res, VisitResult::Continue) {
                return Ok(res);
            }
        }

        Ok(VisitResult::Done)
    }
}

impl<G: AbstractGraph> Index<NodeIndex> for Graph<G> {
    type Output = G::Node;

    fn index(&self, index: NodeIndex) -> &Self::Output {
        &self.digraph[index]
    }
}

impl<G: AbstractGraph> IndexMut<NodeIndex> for Graph<G> {
    fn index_mut(&mut self, index: NodeIndex) -> &mut Self::Output {
        &mut self.digraph[index]
    }
}
