use std::ops::Deref;

use crate::prelude::{
    AnyBuffer, Processor, ProcessorError, ProcessorInputs, ProcessorOutputs, SignalSpec,
};

use super::Graph;

#[derive(Default)]
pub struct SubGraph(Graph);

impl SubGraph {
    /// Creates a new `SubGraph` from the given `Graph`.
    pub fn new(graph: Graph) -> Self {
        Self(graph)
    }
}

impl From<Graph> for SubGraph {
    fn from(graph: Graph) -> Self {
        Self(graph)
    }
}

impl Deref for SubGraph {
    type Target = Graph;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Processor for SubGraph {
    fn name(&self) -> &str {
        "SubGraph"
    }

    fn input_spec(&self) -> Vec<SignalSpec> {
        self.0.with_inner(|graph| {
            graph
                .input_indices()
                .iter()
                .flat_map(|&node_id| graph.graph[node_id].input_spec())
                .cloned()
                .collect()
        })
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        self.0.with_inner(|graph| {
            graph
                .output_indices()
                .iter()
                .flat_map(|&node_id| graph.graph[node_id].output_spec())
                .cloned()
                .collect()
        })
    }

    fn create_output_buffers(&self, size: usize) -> Vec<AnyBuffer> {
        self.0.with_inner(|graph| {
            graph
                .output_indices()
                .iter()
                .flat_map(|&node_id| {
                    graph.graph[node_id]
                        .processor
                        .lock()
                        .create_output_buffers(size)
                })
                .collect()
        })
    }

    fn allocate(&mut self, sample_rate: f32, max_block_size: usize) {
        self.0
            .with_inner(|graph| graph.allocate(sample_rate, max_block_size));
    }

    fn resize_buffers(&mut self, sample_rate: f32, block_size: usize) {
        self.0
            .with_inner(|graph| graph.resize_buffers(sample_rate, block_size));
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        self.0.with_inner(|graph| -> Result<(), ProcessorError> {
            for input_idx in 0..inputs.num_inputs() {
                let input = inputs.input(input_idx);

                if let Some(input) = input {
                    let node_id = graph.input_indices()[input_idx];
                    graph.graph[node_id].outputs[0].clone_from(input);
                }
            }

            graph
                .process()
                .map_err(|e| ProcessorError::SubGraphError(Box::new(e)))?;

            for output_idx in 0..outputs.num_outputs() {
                let mut output = outputs.output(output_idx);

                let node_id = graph.output_indices()[output_idx];
                let output_buffer = &graph.graph[node_id].outputs[0];
                output.clone_from(output_buffer);
            }

            Ok(())
        })?;

        Ok(())
    }
}
