use crate::{
    TypeInfo,
    graph::{Graph, NodeIndex},
};

pub trait Node {
    fn num_inputs(&self) -> usize;
    fn num_outputs(&self) -> usize;
    fn input_type(&self, index: u32) -> Option<TypeInfo>;
    fn output_type(&self, index: u32) -> Option<TypeInfo>;
    fn input_name(&self, index: u32) -> Option<&str>;
    fn output_name(&self, index: u32) -> Option<&str>;
}

pub trait AsNodeInputIndex<N: Node>: Send + ToString + Copy + 'static {
    fn as_node_input_index(&self, graph: &Graph<N>, node: NodeIndex) -> Option<u32>;
}

impl<N: Node> AsNodeInputIndex<N> for u32 {
    fn as_node_input_index(&self, graph: &Graph<N>, node: NodeIndex) -> Option<u32> {
        if *self < graph[node].num_inputs() as u32 {
            Some(*self)
        } else {
            None
        }
    }
}

impl<N: Node> AsNodeInputIndex<N> for &'static str {
    fn as_node_input_index(&self, graph: &Graph<N>, node: NodeIndex) -> Option<u32> {
        for i in 0..graph[node].num_inputs() {
            if let Some(name) = graph[node].input_name(i as u32)
                && &name == self
            {
                return Some(i as u32);
            }
        }
        None
    }
}

pub trait AsNodeOutputIndex<N: Node>: Send + ToString + Copy + 'static {
    fn as_node_output_index(&self, graph: &Graph<N>, node: NodeIndex) -> Option<u32>;
}

impl<N: Node> AsNodeOutputIndex<N> for u32 {
    fn as_node_output_index(&self, graph: &Graph<N>, node: NodeIndex) -> Option<u32> {
        if *self < graph[node].num_outputs() as u32 {
            Some(*self)
        } else {
            None
        }
    }
}

impl<N: Node> AsNodeOutputIndex<N> for &'static str {
    fn as_node_output_index(&self, graph: &Graph<N>, node: NodeIndex) -> Option<u32> {
        for i in 0..graph[node].num_outputs() {
            if let Some(name) = graph[node].output_name(i as u32)
                && &name == self
            {
                return Some(i as u32);
            }
        }
        None
    }
}

pub struct NodeInput<N: Node, I: AsNodeInputIndex<N>> {
    pub node: NodeIndex,
    pub index: I,
    _phantom: std::marker::PhantomData<N>,
}

impl<N: Node, I: AsNodeInputIndex<N>> Clone for NodeInput<N, I> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<N: Node, I: AsNodeInputIndex<N>> Copy for NodeInput<N, I> {}

impl<N: Node, I: AsNodeInputIndex<N>> NodeInput<N, I> {
    pub fn new(node: NodeIndex, index: I) -> Self {
        Self {
            node,
            index,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<N: Node> From<NodeIndex> for NodeInput<N, u32> {
    fn from(node: NodeIndex) -> Self {
        Self::new(node, 0)
    }
}

pub struct NodeOutput<N: Node, I: AsNodeOutputIndex<N>> {
    pub node: NodeIndex,
    pub index: I,
    _phantom: std::marker::PhantomData<N>,
}

pub trait AsNodeOutput<N: Node, I: AsNodeOutputIndex<N>> {
    fn as_node_output(&self, graph: &Graph<N>) -> NodeOutput<N, I>;
}

impl<N: Node, I: AsNodeOutputIndex<N>> Clone for NodeOutput<N, I> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<N: Node, I: AsNodeOutputIndex<N>> Copy for NodeOutput<N, I> {}

impl<N: Node, I: AsNodeOutputIndex<N>> NodeOutput<N, I> {
    pub fn new(node: NodeIndex, index: I) -> Self {
        Self {
            node,
            index,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<N: Node> From<NodeIndex> for NodeOutput<N, u32> {
    fn from(node: NodeIndex) -> Self {
        Self::new(node, 0)
    }
}

pub trait NodeIndexExt<N: Node> {
    fn input<I: AsNodeInputIndex<N>>(&self, index: I) -> NodeInput<N, I>;

    fn output<I: AsNodeOutputIndex<N>>(&self, index: I) -> NodeOutput<N, I>;
}

impl<N: Node> NodeIndexExt<N> for NodeIndex {
    fn input<I: AsNodeInputIndex<N>>(&self, index: I) -> NodeInput<N, I> {
        NodeInput::new(*self, index)
    }

    fn output<I: AsNodeOutputIndex<N>>(&self, index: I) -> NodeOutput<N, I> {
        NodeOutput::new(*self, index)
    }
}
