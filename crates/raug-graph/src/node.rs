use crate::{
    TypeInfo,
    graph::{Graph, NodeIndex},
};

pub trait Node {
    fn name(&self) -> Option<String> {
        None
    }
    fn num_inputs(&self) -> usize;
    fn num_outputs(&self) -> usize;
    fn input_type(&self, index: u32) -> Option<TypeInfo>;
    fn output_type(&self, index: u32) -> Option<TypeInfo>;
    fn input_name(&self, index: u32) -> Option<&str>;
    fn output_name(&self, index: u32) -> Option<&str>;
}

pub trait AsNodeInputIndex<N: Node>: ToString {
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

impl<N: Node> AsNodeInputIndex<N> for &str {
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

pub trait AsNodeOutputIndex<N: Node>: ToString {
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

impl<N: Node> AsNodeOutputIndex<N> for &str {
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
