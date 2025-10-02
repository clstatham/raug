use crate::prelude::{
    AnyBuffer, Processor, ProcessorError, ProcessorInputs, ProcessorOutputs, SignalSpec,
};

use super::Graph;

impl Processor for Graph {
    fn name(&self) -> &str {
        "Graph"
    }

    fn input_spec(&self) -> Vec<SignalSpec> {
        self.input_indices()
            .iter()
            .flat_map(|&node_id| self.graph[node_id].input_spec())
            .cloned()
            .collect()
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        self.output_indices()
            .iter()
            .flat_map(|&node_id| self.graph[node_id].output_spec())
            .cloned()
            .collect()
    }

    fn create_output_buffers(&self, size: usize) -> Vec<AnyBuffer> {
        self.output_indices()
            .iter()
            .flat_map(|&node_id| {
                self.graph[node_id]
                    .processor
                    .lock()
                    .create_output_buffers(size)
            })
            .collect()
    }

    fn allocate(&mut self, sample_rate: f32, max_block_size: usize) {
        Graph::allocate(self, sample_rate, max_block_size);
    }

    fn resize_buffers(&mut self, sample_rate: f32, block_size: usize) {
        Graph::resize_buffers(self, sample_rate, block_size);
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for input_idx in 0..inputs.num_inputs() {
            let input = inputs.input(input_idx);

            if let Some(input) = input {
                let node_id = self.input_indices()[input_idx];
                self.graph[node_id].outputs[0].clone_from(input);
            }
        }

        self.process()
            .map_err(|e| ProcessorError::SubGraphError(Box::new(e)))?;

        for output_idx in 0..outputs.num_outputs() {
            let mut output = outputs.output(output_idx);

            let node_id = self.output_indices()[output_idx];
            let output_buffer = &self.graph[node_id].outputs[0];
            output.clone_from(output_buffer);
        }
        Ok(())
    }
}
