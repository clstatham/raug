use std::{
    fmt::{Debug, Display},
    sync::{Arc, Weak},
};

use parking_lot::Mutex;

use crate::{
    Error, TypeInfo,
    graph::{AbstractGraph, NodeIndex},
    node::AbstractNode,
};

pub struct GraphBuilder<G: AbstractGraph> {
    inner: Arc<Mutex<G>>,
}

impl<G: AbstractGraph + Default> Default for GraphBuilder<G> {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(G::default())),
        }
    }
}

impl<G: AbstractGraph> Clone for GraphBuilder<G> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<G: AbstractGraph> GraphBuilder<G> {
    pub fn is_same_graph(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }

    pub fn downgrade(&self) -> WeakGraphBuilder<G> {
        WeakGraphBuilder {
            inner: Arc::downgrade(&self.inner),
        }
    }

    pub fn from_inner(inner: G) -> Self {
        Self {
            inner: Arc::new(Mutex::new(inner)),
        }
    }

    pub fn try_into_inner(self) -> Result<G, Self> {
        match Arc::try_unwrap(self.inner) {
            Ok(inner) => Ok(inner.into_inner()),
            Err(arc) => Err(Self { inner: arc }),
        }
    }

    pub fn inner(&self) -> &Arc<Mutex<G>> {
        &self.inner
    }

    pub fn with_inner<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut G) -> R,
    {
        let mut inner = self.inner.lock();
        f(&mut inner)
    }

    pub fn input_count(&self) -> usize {
        self.with_inner(|graph| graph.graph().inputs().len())
    }

    pub fn output_count(&self) -> usize {
        self.with_inner(|graph| graph.graph().outputs().len())
    }

    pub fn node_count(&self) -> usize {
        self.with_inner(|graph| graph.graph().digraph().node_count())
    }

    pub fn edge_count(&self) -> usize {
        self.with_inner(|graph| graph.graph().digraph().edge_count())
    }

    pub fn add_input(&self, node: G::Node) -> NodeBuilder<G> {
        let node_id = self.with_inner(|graph| graph.graph_mut().add_input(node));
        NodeBuilder::new(self.clone(), node_id)
    }

    pub fn add_output(&self, node: G::Node) -> NodeBuilder<G> {
        let node_id = self.with_inner(|graph| graph.graph_mut().add_output(node));
        NodeBuilder::new(self.clone(), node_id)
    }

    pub fn add_node(&self, node: G::Node) -> NodeBuilder<G> {
        let node_id = self.with_inner(|graph| graph.graph_mut().add_node(node));
        NodeBuilder::new(self.clone(), node_id)
    }

    pub fn connect(
        &self,
        source: impl IntoNode<G>,
        source_output: impl IntoIndex,
        target: impl IntoNode<G>,
        target_input: impl IntoIndex,
    ) -> Result<(), Error> {
        let source_node = source.into_node(self);
        let target_node = target.into_node(self);
        let source_output_index = source_output
            .into_output_idx::<G>(&source_node)
            .ok_or_else(|| Error::OutputIndexOutOfBounds {
                node: source_node.id(),
                index: format!("{source_output:?}"),
                num_outputs: source_node.num_outputs(),
            })?;
        let target_input_index =
            target_input
                .into_input_idx::<G>(&target_node)
                .ok_or_else(|| Error::InputIndexOutOfBounds {
                    node: target_node.id(),
                    index: format!("{target_input:?}"),
                    num_inputs: target_node.num_inputs(),
                })?;

        self.with_inner(|graph| {
            graph.graph_mut().connect(
                source_node.id(),
                source_output_index,
                target_node.id(),
                target_input_index,
            )
        })?;

        Ok(())
    }
}

pub struct WeakGraphBuilder<G: AbstractGraph> {
    inner: Weak<Mutex<G>>,
}

impl<G: AbstractGraph> WeakGraphBuilder<G> {
    pub fn upgrade(&self) -> Option<GraphBuilder<G>> {
        self.inner.upgrade().map(|inner| GraphBuilder { inner })
    }

    pub fn is_same_graph(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.inner, &other.inner)
    }
}

impl<G: AbstractGraph> Clone for WeakGraphBuilder<G> {
    fn clone(&self) -> Self {
        Self {
            inner: Weak::clone(&self.inner),
        }
    }
}

pub struct NodeBuilder<G: AbstractGraph> {
    graph: WeakGraphBuilder<G>,
    node_id: NodeIndex,
}

impl<G: AbstractGraph> Clone for NodeBuilder<G> {
    fn clone(&self) -> Self {
        Self {
            graph: self.graph.clone(),
            node_id: self.node_id,
        }
    }
}

impl<G: AbstractGraph> Debug for NodeBuilder<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Node({:?})", self.node_id)
    }
}

impl<G: AbstractGraph> Display for NodeBuilder<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(name) = self.name() {
            write!(f, "{}", name)
        } else {
            write!(f, "Node({:?})", self.node_id)
        }
    }
}

impl<G: AbstractGraph> NodeBuilder<G> {
    pub fn new(graph: GraphBuilder<G>, node_id: NodeIndex) -> Self {
        let has_node = graph.with_inner(|g| g.graph().digraph().node_weight(node_id).is_some());
        assert!(
            has_node,
            "NodeBuilder::new called with a node_id that does not exist in the graph"
        );
        Self {
            graph: graph.downgrade(),
            node_id,
        }
    }

    pub fn is_same_node(&self, other: &Self) -> bool {
        self.graph.is_same_graph(&other.graph) && self.node_id == other.node_id
    }

    pub fn id(&self) -> NodeIndex {
        self.node_id
    }

    #[track_caller]
    pub fn graph(&self) -> GraphBuilder<G> {
        self.graph.upgrade().expect("GraphBuilder dropped")
    }

    #[track_caller]
    pub fn with_inner<F, R>(&self, mut f: F) -> R
    where
        F: FnMut(&G::Node) -> R,
    {
        self.graph()
            .with_inner(|graph| f(&graph.graph()[self.node_id]))
    }

    #[track_caller]
    pub fn with_inner_mut<F, R>(&self, mut f: F) -> R
    where
        F: FnMut(&mut G::Node) -> R,
    {
        self.graph()
            .with_inner(|graph| f(&mut graph.graph_mut()[self.node_id]))
    }

    #[track_caller]
    pub fn name(&self) -> Option<String> {
        self.with_inner(|node| node.name())
    }

    #[track_caller]
    pub fn num_inputs(&self) -> usize {
        self.with_inner(|node| node.num_inputs())
    }

    #[track_caller]
    pub fn num_outputs(&self) -> usize {
        self.with_inner(|node| node.num_outputs())
    }

    #[track_caller]
    pub fn input_name(&self, index: impl IntoIndex) -> Option<String> {
        let index = index.into_input_idx(self)?;
        self.with_inner(|node| node.input_name(index).map(ToOwned::to_owned))
    }

    #[track_caller]
    pub fn output_name(&self, index: impl IntoIndex) -> Option<String> {
        let index = index.into_output_idx(self)?;
        self.with_inner(|node| node.output_name(index).map(ToOwned::to_owned))
    }

    #[track_caller]
    pub fn input_type(&self, index: impl IntoIndex) -> Option<TypeInfo> {
        let index = index.into_input_idx(self)?;
        self.with_inner(|node| node.input_type(index))
    }

    #[track_caller]
    pub fn output_type(&self, index: impl IntoIndex) -> Option<TypeInfo> {
        let index = index.into_output_idx(self)?;
        self.with_inner(|node| node.output_type(index))
    }

    #[track_caller]
    pub fn input(&self, index: impl IntoIndex) -> Result<Input<G>, Error> {
        let input_index =
            index
                .into_input_idx::<G>(self)
                .ok_or_else(|| Error::InputIndexOutOfBounds {
                    node: self.node_id,
                    index: format!("{index:?}"),
                    num_inputs: self.num_inputs(),
                })?;

        Ok(Input {
            node: self.clone(),
            input_index,
        })
    }

    #[track_caller]
    pub fn output(&self, index: impl IntoIndex) -> Result<Output<G>, Error> {
        let output_index =
            index
                .into_output_idx::<G>(self)
                .ok_or_else(|| Error::OutputIndexOutOfBounds {
                    node: self.node_id,
                    index: format!("{index:?}"),
                    num_outputs: self.num_outputs(),
                })?;

        Ok(Output {
            node: self.clone(),
            output_index,
        })
    }

    pub fn disconnect_all_inputs(&self) {
        self.graph().with_inner(|graph| {
            graph.graph_mut().disconnect_all_inputs(self.node_id);
        })
    }

    pub fn disconnect_all_outputs(&self) {
        self.graph().with_inner(|graph| {
            graph.graph_mut().disconnect_all_outputs(self.node_id);
        })
    }

    pub fn disconnect_all(&self) {
        self.graph().with_inner(|graph| {
            graph.graph_mut().disconnect_all(self.node_id);
        })
    }
}

pub trait IntoIndex: Debug + Copy {
    fn into_input_idx<G: AbstractGraph>(self, node: &NodeBuilder<G>) -> Option<u32>;
    fn into_output_idx<G: AbstractGraph>(self, node: &NodeBuilder<G>) -> Option<u32>;
}

impl IntoIndex for u32 {
    fn into_input_idx<G: AbstractGraph>(self, node: &NodeBuilder<G>) -> Option<u32> {
        if self < node.num_inputs() as u32 {
            Some(self)
        } else {
            None
        }
    }

    fn into_output_idx<G: AbstractGraph>(self, node: &NodeBuilder<G>) -> Option<u32> {
        if self < node.num_outputs() as u32 {
            Some(self)
        } else {
            None
        }
    }
}

impl IntoIndex for usize {
    fn into_input_idx<G: AbstractGraph>(self, node: &NodeBuilder<G>) -> Option<u32> {
        if self < node.num_inputs() {
            Some(self as u32)
        } else {
            None
        }
    }

    fn into_output_idx<G: AbstractGraph>(self, node: &NodeBuilder<G>) -> Option<u32> {
        if self < node.num_outputs() {
            Some(self as u32)
        } else {
            None
        }
    }
}

impl IntoIndex for i32 {
    fn into_input_idx<G: AbstractGraph>(self, node: &NodeBuilder<G>) -> Option<u32> {
        if self >= 0 && (self as usize) < node.num_inputs() {
            Some(self as u32)
        } else {
            None
        }
    }

    fn into_output_idx<G: AbstractGraph>(self, node: &NodeBuilder<G>) -> Option<u32> {
        if self >= 0 && (self as usize) < node.num_outputs() {
            Some(self as u32)
        } else {
            None
        }
    }
}

impl IntoIndex for &str {
    fn into_input_idx<G: AbstractGraph>(self, node: &NodeBuilder<G>) -> Option<u32> {
        for i in 0..node.num_inputs() {
            if node.input_name(i as u32).as_deref() == Some(self) {
                return Some(i as u32);
            }
        }
        None
    }

    fn into_output_idx<G: AbstractGraph>(self, node: &NodeBuilder<G>) -> Option<u32> {
        for i in 0..node.num_outputs() {
            if node.output_name(i as u32).as_deref() == Some(self) {
                return Some(i as u32);
            }
        }
        None
    }
}

pub struct Input<G: AbstractGraph> {
    pub node: NodeBuilder<G>,
    pub input_index: u32,
}

impl<G: AbstractGraph> Input<G> {
    pub fn node_id(&self) -> NodeIndex {
        self.node.id()
    }

    pub fn index(&self) -> u32 {
        self.input_index
    }

    pub fn type_info(&self) -> Option<TypeInfo> {
        self.node
            .with_inner(|node| node.input_type(self.input_index))
    }

    pub fn name(&self) -> Option<String> {
        self.node.input_name(self.input_index)
    }

    pub fn node(&self) -> NodeBuilder<G> {
        self.node.clone()
    }

    pub fn graph(&self) -> GraphBuilder<G> {
        self.node.graph()
    }

    pub fn connect(&self, output: impl IntoOutput<G>) -> Result<(), Error> {
        let output = output.into_output(&self.graph())?;
        self.graph().connect(
            output.node.id(),
            output.output_index,
            self.node.id(),
            self.input_index,
        )?;
        Ok(())
    }
}

impl<G: AbstractGraph> Clone for Input<G> {
    fn clone(&self) -> Self {
        Self {
            node: self.node.clone(),
            input_index: self.input_index,
        }
    }
}

impl<G: AbstractGraph> Debug for Input<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Input({:?}, {})", self.node.id(), self.input_index)
    }
}

impl<G: AbstractGraph> Display for Input<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(name) = self.node.input_name(self.input_index) {
            write!(f, "{}.{}", self.node, name)
        } else {
            write!(f, "{}.{}", self.node, self.input_index)
        }
    }
}

pub struct Output<G: AbstractGraph> {
    pub node: NodeBuilder<G>,
    pub output_index: u32,
}

impl<G: AbstractGraph> Output<G> {
    pub fn node_id(&self) -> NodeIndex {
        self.node.id()
    }

    pub fn index(&self) -> u32 {
        self.output_index
    }

    pub fn type_info(&self) -> Option<TypeInfo> {
        self.node
            .with_inner(|node| node.output_type(self.output_index))
    }

    pub fn name(&self) -> Option<String> {
        self.node.output_name(self.output_index)
    }
}

impl<G: AbstractGraph> Clone for Output<G> {
    fn clone(&self) -> Self {
        Self {
            node: self.node.clone(),
            output_index: self.output_index,
        }
    }
}

impl<G: AbstractGraph> Debug for Output<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Output({:?}, {})", self.node.id(), self.output_index)
    }
}

impl<G: AbstractGraph> Display for Output<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(name) = self.node.output_name(self.output_index) {
            write!(f, "{}.{}", self.node, name)
        } else {
            write!(f, "{}.{}", self.node, self.output_index)
        }
    }
}

pub trait IntoNode<G: AbstractGraph> {
    fn into_node(self, graph: &GraphBuilder<G>) -> NodeBuilder<G>;
}

impl<G: AbstractGraph> IntoNode<G> for NodeBuilder<G> {
    fn into_node(self, graph: &GraphBuilder<G>) -> NodeBuilder<G> {
        assert!(
            graph.downgrade().is_same_graph(&self.graph),
            "Cannot convert NodeBuilder from a different graph"
        );
        self
    }
}

impl<G: AbstractGraph> IntoNode<G> for NodeIndex {
    fn into_node(self, graph: &GraphBuilder<G>) -> NodeBuilder<G> {
        NodeBuilder::new(graph.clone(), self)
    }
}

impl<G: AbstractGraph> IntoNode<G> for &NodeBuilder<G> {
    fn into_node(self, graph: &GraphBuilder<G>) -> NodeBuilder<G> {
        assert!(
            graph.downgrade().is_same_graph(&self.graph),
            "Cannot convert NodeBuilder from a different graph"
        );
        self.clone()
    }
}

pub trait IntoInput<G: AbstractGraph> {
    fn into_input(self, graph: &GraphBuilder<G>) -> Result<Input<G>, Error>;
}

pub trait IntoOutput<G: AbstractGraph> {
    fn into_output(self, graph: &GraphBuilder<G>) -> Result<Output<G>, Error>;
}

impl<G: AbstractGraph> IntoInput<G> for Input<G> {
    fn into_input(self, graph: &GraphBuilder<G>) -> Result<Input<G>, Error> {
        assert!(
            graph.downgrade().is_same_graph(&self.node.graph),
            "Cannot convert Input from a different graph"
        );
        Ok(self)
    }
}

impl<G: AbstractGraph> IntoOutput<G> for Output<G> {
    fn into_output(self, graph: &GraphBuilder<G>) -> Result<Output<G>, Error> {
        assert!(
            graph.downgrade().is_same_graph(&self.node.graph),
            "Cannot convert Output from a different graph"
        );
        Ok(self)
    }
}
