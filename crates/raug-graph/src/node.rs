use crate::{
    TypeInfo,
    graph::{AddConnections, Graph},
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

pub trait NodeIndexExt<N: Node>: Copy {
    fn and_connect(self, graph: &mut Graph<N>) -> AddConnections<'_, N>
    where
        Self: Sized;
}

impl<N: Node> NodeIndexExt<N> for crate::graph::NodeIndex {
    fn and_connect(self, graph: &mut Graph<N>) -> AddConnections<'_, N> {
        AddConnections::new(graph, self)
    }
}
