use wasm_bindgen::prelude::*;

mod builtins;

use raug::prelude::RaugNodeIndexExt;

#[wasm_bindgen]
extern "C" {
    pub fn alert(s: &str);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[wasm_bindgen]
#[repr(transparent)]
pub struct Node {
    pub(crate) inner: raug::prelude::Node,
}

#[wasm_bindgen]
impl Node {
    #[wasm_bindgen(js_name = "input")]
    pub fn input(&self, index: u32) -> Input {
        Input {
            inner: self.inner.input(index),
        }
    }

    #[wasm_bindgen(js_name = "output")]
    pub fn output(&self, index: u32) -> Output {
        Output {
            inner: self.inner.output(index),
        }
    }
}

#[wasm_bindgen]
#[repr(transparent)]
pub struct Output {
    pub(crate) inner: raug::graph::node::Output<u32>,
}

#[wasm_bindgen]
#[repr(transparent)]
pub struct Input {
    pub(crate) inner: raug::graph::node::Input<u32>,
}

#[derive(Default)]
#[wasm_bindgen]
pub struct Graph {
    pub(crate) inner: raug::prelude::Graph,
    pub(crate) interleaved_output: Vec<f32>,
}

#[wasm_bindgen(js_name = "getMemory")]
pub fn get_memory() -> JsValue {
    // Access the WebAssembly memory
    wasm_bindgen::memory()
}

#[wasm_bindgen]
impl Graph {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: raug::prelude::Graph::new(),
            interleaved_output: Vec::new(),
        }
    }

    #[wasm_bindgen(js_name = "connectRaw")]
    pub fn connect_raw(
        &mut self,
        from_node: Node,
        from_index: u32,
        to_node: Node,
        to_index: u32,
    ) -> Result<(), JsValue> {
        self.inner.connect(
            from_node.inner.output(from_index),
            to_node.inner.input(to_index),
        );
        Ok(())
    }

    #[wasm_bindgen(js_name = "connect")]
    pub fn connect(&mut self, from: Output, to: Input) -> Result<(), JsValue> {
        self.inner.connect(from.inner, to.inner);
        Ok(())
    }

    #[wasm_bindgen(js_name = "disconnectRaw")]
    pub fn disconnect_raw(&mut self, to_node: Node, to_index: u32) -> Result<(), JsValue> {
        self.inner.disconnect(to_node.inner.input(to_index));
        Ok(())
    }

    #[wasm_bindgen(js_name = "disconnect")]
    pub fn disconnect(&mut self, to: Input) -> Result<(), JsValue> {
        self.inner.disconnect(to.inner);
        Ok(())
    }

    #[wasm_bindgen(js_name = "connectConstant")]
    pub fn connect_constant(&mut self, value: f32, to: Input) -> Result<(), JsValue> {
        self.inner.connect_constant(value, to.inner);
        Ok(())
    }

    #[wasm_bindgen(js_name = "connectAudioOutput")]
    pub fn connect_audio_output(&mut self, from: Output) -> Result<(), JsValue> {
        self.inner.connect_audio_output(from.inner);
        Ok(())
    }

    #[wasm_bindgen(js_name = "nodeCount")]
    pub fn node_count(&self) -> usize {
        self.inner.node_count()
    }

    #[wasm_bindgen(js_name = "numAudioOutputs")]
    pub fn num_audio_outputs(&self) -> usize {
        self.inner.num_audio_outputs()
    }

    #[wasm_bindgen(js_name = "getOutput")]
    pub fn get_output(&self, index: u32) -> Result<Vec<f32>, JsValue> {
        let output = self
            .inner
            .get_output(index as usize)
            .ok_or_else(|| JsValue::from_str("Output index out of range"))?;
        Ok(output.to_vec())
    }

    #[wasm_bindgen(js_name = "allocate")]
    pub fn allocate(&mut self, sample_rate: f32, block_size: usize) -> Result<(), JsValue> {
        self.inner.allocate(sample_rate, block_size);

        let num_outputs = self.inner.num_audio_outputs();
        self.interleaved_output
            .resize(num_outputs * block_size, 0.0);
        Ok(())
    }

    #[wasm_bindgen(js_name = "resizeBuffers")]
    pub fn resize_buffers(&mut self, block_size: usize) -> Result<(), JsValue> {
        self.inner
            .resize_buffers(self.inner.sample_rate(), block_size);

        let num_outputs = self.inner.num_audio_outputs();
        self.interleaved_output
            .resize(num_outputs * block_size, 0.0);
        Ok(())
    }

    #[wasm_bindgen(js_name = "process")]
    pub fn process(&mut self) -> Result<(), JsValue> {
        self.inner
            .process()
            .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;

        let num_outputs = self.inner.num_audio_outputs();
        let block_size = self.inner.block_size();
        for c in 0..num_outputs {
            let output = self.inner.get_output(c).unwrap();
            for i in 0..block_size {
                self.interleaved_output[i * num_outputs + c] = output[i];
            }
        }

        Ok(())
    }

    #[wasm_bindgen(js_name = "outputBufferPtr")]
    pub fn output_buffer_ptr(&mut self) -> Result<*const f32, JsValue> {
        Ok(self.interleaved_output.as_ptr())
    }
}

#[wasm_bindgen(start)]
pub fn main_js() {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();
}
