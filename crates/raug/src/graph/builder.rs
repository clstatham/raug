use raug_graph::node::{AsNodeInputIndex, AsNodeOutputIndex};
use rustc_hash::FxHashMap;

use crate::{
    graph::{
        Graph, Node,
        node::{BuildOnGraph, RaugNodeIndexExt},
        playback::GraphRunResult,
    },
    prelude::{AudioOut, KillSwitch, Param, ProcessorNode},
};

/// Method-chain style builder for constructing a `Graph`.
#[derive(Default)]
pub struct GraphBuilder {
    nodes: FxHashMap<String, Node>,
    params: FxHashMap<String, Param<f32>>,
    pub(crate) graph: Graph,
}

impl GraphBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_graph(graph: Graph) -> Self {
        Self {
            graph,
            ..Default::default()
        }
    }

    #[track_caller]
    pub(crate) fn get(&self, name: &str) -> Node {
        *self
            .nodes
            .get(name)
            .unwrap_or_else(|| panic!("unknown node name: {}", name))
    }

    #[track_caller]
    pub(crate) fn get_param(&self, name: &str) -> &Param<f32> {
        self.params
            .get(name)
            .unwrap_or_else(|| panic!("node is not a param: {}", name))
    }

    pub fn param_iter(&self) -> impl Iterator<Item = (&String, &Param<f32>)> {
        self.params.iter()
    }

    pub fn finish(self) -> Graph {
        self.graph
    }

    #[track_caller]
    pub fn insert_node(mut self, name: &str, node: impl BuildOnGraph) -> Self {
        assert!(!self.nodes.contains_key(name), "node name already used");
        let node = self.graph.node(node);
        self.nodes.insert(name.to_string(), node);
        self
    }

    pub fn insert_param(mut self, name: &str, initial_value: f32) -> Self {
        assert!(!self.nodes.contains_key(name), "node name already used");
        let (node, param) = self.graph.param(initial_value);
        self.nodes.insert(name.to_string(), node);
        self.params.insert(name.to_string(), param);
        self
    }

    #[track_caller]
    pub fn connect<I, O>(mut self, from: &str, output: O, to: &str, input: I) -> Self
    where
        I: AsNodeInputIndex<ProcessorNode>,
        O: AsNodeOutputIndex<ProcessorNode>,
    {
        let from = self.get(from);
        let to = self.get(to);
        self.graph.connect(from.output(output), to.input(input));
        self
    }

    #[track_caller]
    pub fn connect_constant<I>(mut self, value: f32, to: &str, input: I) -> Self
    where
        I: AsNodeInputIndex<ProcessorNode>,
    {
        let to = self.get(to);
        self.graph.connect_constant(value, to.input(input));
        self
    }

    #[track_caller]
    pub fn connect_audio_output(mut self, from: &str) -> Self {
        let from = self.get(from);
        self.graph.connect_audio_output(from);
        self
    }

    #[track_caller]
    pub fn connect_param<I>(
        mut self,
        param_name: &str,
        initial_value: f32,
        to: &str,
        input: I,
    ) -> Self
    where
        I: AsNodeInputIndex<ProcessorNode>,
    {
        assert!(
            !self.params.contains_key(param_name),
            "param name already used"
        );
        let (node, param) = self.graph.param(initial_value);
        self.params.insert(param_name.to_string(), param);
        self.nodes.insert(param_name.to_string(), node);
        let to = self.get(to);
        self.graph.connect(node, to.input(input));
        self
    }

    #[cfg(feature = "expr")]
    #[track_caller]
    pub fn connect_expr<I>(mut self, node: &str, node_input: I, expr: &str) -> Self
    where
        I: AsNodeInputIndex<ProcessorNode>,
    {
        use crate::graph::expr::{Ast, Val};

        let ast = Ast::parse(expr).unwrap_or_else(|e| panic!("expression parsing failed: {}", e));
        let val = ast.eval(&mut self).expect("expression evaluation failed");
        let to = self.get(node);
        match val {
            Val::Number(n) => {
                self.graph.connect_constant(n, to.input(node_input));
            }
            Val::NodePort(node, port) => {
                self.graph.connect(node.output(port), to.input(node_input));
            }
            Val::Void => panic!("expression evaluated to void"),
        }

        self
    }

    #[cfg(feature = "expr")]
    #[track_caller]
    pub fn eval_expr(mut self, expr: &str) -> Self {
        use crate::graph::expr::{Ast, Val};

        let ast = Ast::parse(expr).unwrap_or_else(|e| panic!("expression parsing failed: {}", e));
        let val = ast.eval(&mut self).expect("expression evaluation failed");
        assert!(
            matches!(val, Val::Void),
            "eval_expr can only be used for expressions that evaluate to void"
        );
        self
    }

    pub fn play(
        mut self,
        output_stream: &impl AudioOut,
        kill_switch: Option<KillSwitch>,
    ) -> GraphRunResult<()> {
        self.graph.play(output_stream, kill_switch)
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::ProcResult;

    use super::*;
    use raug_macros::processor;

    #[processor(derive(Default))]
    fn sine_oscillator(#[input] freq: &f32, #[output] out: &mut f32) -> ProcResult<()> {
        *out = (*freq * 2.0 * std::f32::consts::PI).sin(); // Dummy implementation
        Ok(())
    }

    #[test]
    fn test_builder() {
        let graph = GraphBuilder::new()
            .insert_node("osc", SineOscillator::default())
            .insert_param("freq", 440.0)
            .connect("freq", 0, "osc", "freq")
            .connect_audio_output("osc")
            .finish();
        assert_eq!(graph.node_count(), 3); // osc, freq param, audio output
    }

    #[test]
    #[should_panic(expected = "node name already used")]
    fn test_duplicate_node_name() {
        let _graph = GraphBuilder::new()
            .insert_node("osc", SineOscillator::default())
            .insert_node("osc", SineOscillator::default()) // Duplicate name
            .finish();
    }

    #[cfg(feature = "expr")]
    #[test]
    fn test_builder_with_expr() {
        let graph = GraphBuilder::new()
            .insert_node("osc", SineOscillator::default())
            .insert_param("freq", 440.0)
            .connect_expr("osc", "freq", "440.0 * 2.0")
            .connect_audio_output("osc")
            .finish();
        assert_eq!(graph.node_count(), 4); // osc, freq param, const node (440.0 * 2.0 pre-calculated), audio output
    }
}
