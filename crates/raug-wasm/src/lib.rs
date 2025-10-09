use wasm_bindgen::prelude::*;

mod builtins;

use raug::prelude::RaugNodeIndexExt;

use crate::builtins::Proc;

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
    #[wasm_bindgen(constructor)]
    pub fn new(index: u32) -> Self {
        Self {
            inner: raug::prelude::Node::new(index as usize),
        }
    }

    #[wasm_bindgen(js_name = "id")]
    pub fn id(&self) -> u32 {
        self.inner.index() as u32
    }

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[wasm_bindgen]
#[repr(transparent)]
pub struct Edge {
    pub(crate) inner: raug::graph::Connection,
}

#[wasm_bindgen]
impl Edge {
    #[wasm_bindgen(js_name = "sourceNode")]
    pub fn source_node(&self) -> Node {
        Node {
            inner: self.inner.source.into(),
        }
    }

    #[wasm_bindgen(js_name = "sourceOutputIndex")]
    pub fn source_output_index(&self) -> u32 {
        self.inner.source_output
    }

    #[wasm_bindgen(js_name = "targetNode")]
    pub fn target_node(&self) -> Node {
        Node {
            inner: self.inner.target.into(),
        }
    }

    #[wasm_bindgen(js_name = "targetInputIndex")]
    pub fn target_input_index(&self) -> u32 {
        self.inner.target_input
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

#[wasm_bindgen]
#[repr(transparent)]
pub struct FloatParam {
    pub(crate) inner: raug::prelude::Param<f32>,
}

#[wasm_bindgen]
impl FloatParam {
    #[wasm_bindgen(js_name = "set")]
    pub fn set(&self, value: f32) {
        self.inner.set(value);
    }

    #[wasm_bindgen(js_name = "get")]
    pub fn get(&self) -> f32 {
        self.inner.get()
    }
}

#[wasm_bindgen]
#[repr(transparent)]
pub struct BoolParam {
    pub(crate) inner: raug::prelude::Param<bool>,
}

#[wasm_bindgen]
impl BoolParam {
    #[wasm_bindgen(js_name = "set")]
    pub fn set(&self, value: bool) {
        self.inner.set(value);
    }

    #[wasm_bindgen(js_name = "get")]
    pub fn get(&self) -> bool {
        self.inner.get()
    }
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

    #[wasm_bindgen(js_name = "allNodes")]
    pub fn all_nodes(&self) -> js_sys::Array {
        self.inner
            .node_id_iter()
            .map(|node| JsValue::from(Node { inner: node }))
            .collect()
    }

    #[wasm_bindgen(js_name = "allAudioOutputs")]
    pub fn all_audio_outputs(&self) -> js_sys::Array {
        self.inner
            .output_indices()
            .map(|node| JsValue::from(Node { inner: node }))
            .collect()
    }

    #[wasm_bindgen(js_name = "hasNode")]
    pub fn has_node(&self, node: &Node) -> bool {
        self.inner.get_node(node.inner.index()).is_some()
    }

    #[wasm_bindgen(js_name = "nodeName")]
    pub fn node_name(&self, node: &Node) -> Option<String> {
        Some(self.inner.get_node(node.inner.index())?.name().to_string())
    }

    #[wasm_bindgen(js_name = "nodeInputNames")]
    pub fn node_input_names(&self, node: &Node) -> Option<js_sys::Array> {
        let node = self.inner.get_node(node.inner.index())?;
        Some(
            node.input_spec()
                .iter()
                .map(|spec| JsValue::from(spec.name.clone()))
                .collect(),
        )
    }

    #[wasm_bindgen(js_name = "nodeOutputNames")]
    pub fn node_output_names(&self, node: &Node) -> Option<js_sys::Array> {
        let node = self.inner.get_node(node.inner.index())?;
        Some(
            node.output_spec()
                .iter()
                .map(|spec| JsValue::from(spec.name.clone()))
                .collect(),
        )
    }

    #[wasm_bindgen(js_name = "allEdges")]
    pub fn all_edges(&self) -> js_sys::Array {
        self.inner
            .edge_iter()
            .map(|edge| JsValue::from(Edge { inner: *edge }))
            .collect()
    }

    #[wasm_bindgen(js_name = "floatParam")]
    pub fn float_param(&mut self, value: f32) -> Node {
        let (node, _) = self.inner.param(value);
        Node { inner: node }
    }

    #[wasm_bindgen(js_name = "boolParam")]
    pub fn bool_param(&mut self, value: bool) -> Node {
        let (node, _) = self.inner.param(value);
        Node { inner: node }
    }

    #[wasm_bindgen(js_name = "isFloatParam")]
    pub fn is_float_param(&self, node: &Node) -> Option<bool> {
        Some(
            self.inner
                .get_node(node.inner.index())?
                .processor_is::<raug::prelude::Param<f32>>(),
        )
    }

    #[wasm_bindgen(js_name = "isBoolParam")]
    pub fn is_bool_param(&self, node: &Node) -> Option<bool> {
        Some(
            self.inner
                .get_node(node.inner.index())?
                .processor_is::<raug::prelude::Param<bool>>(),
        )
    }

    #[wasm_bindgen(js_name = "getFloatParam")]
    pub fn get_float_param(&self, node: &Node) -> Result<FloatParam, JsValue> {
        let proc = self
            .inner
            .get_node(node.inner.index())
            .ok_or_else(|| JsValue::from_str("Node not found"))?
            .processor_as::<raug::prelude::Param<f32>>()
            .ok_or_else(|| JsValue::from_str("Node is not a float param"))?;
        Ok(FloatParam {
            inner: proc.clone(),
        })
    }

    #[wasm_bindgen(js_name = "getBoolParam")]
    pub fn get_bool_param(&self, node: &Node) -> Result<BoolParam, JsValue> {
        let proc = self
            .inner
            .get_node(node.inner.index())
            .ok_or_else(|| JsValue::from_str("Node not found"))?
            .processor_as::<raug::prelude::Param<bool>>()
            .ok_or_else(|| JsValue::from_str("Node is not a bool param"))?;
        Ok(BoolParam {
            inner: proc.clone(),
        })
    }

    #[wasm_bindgen(js_name = "addNode")]
    pub fn add_node(&mut self, proc: Proc) -> Result<Node, JsValue> {
        let node = self.inner.processor_boxed(proc.inner);
        Ok(Node { inner: node })
    }

    #[wasm_bindgen(js_name = "connectRaw")]
    pub fn connect_raw(
        &mut self,
        from_node: &Node,
        from_index: u32,
        to_node: &Node,
        to_index: u32,
    ) -> Result<Edge, JsValue> {
        self.inner.connect(
            from_node.inner.output(from_index),
            to_node.inner.input(to_index),
        );

        Ok(Edge {
            inner: raug::graph::Connection {
                source: from_node.inner.into(),
                source_output: from_index,
                target: to_node.inner.into(),
                target_input: to_index,
            },
        })
    }

    #[wasm_bindgen(js_name = "connect")]
    pub fn connect(&mut self, from: &Output, to: &Input) -> Result<(), JsValue> {
        self.inner.connect(from.inner, to.inner);
        Ok(())
    }

    #[wasm_bindgen(js_name = "disconnectRaw")]
    pub fn disconnect_raw(
        &mut self,
        from_node: &Node,
        from_index: u32,
        to_node: &Node,
        to_index: u32,
    ) -> Result<Edge, JsValue> {
        self.inner.disconnect(
            from_node.inner.output(from_index),
            to_node.inner.input(to_index),
        );
        Ok(Edge {
            inner: raug::graph::Connection {
                source: from_node.inner.into(),
                source_output: from_index,
                target: to_node.inner.into(),
                target_input: to_index,
            },
        })
    }

    #[wasm_bindgen(js_name = "disconnect")]
    pub fn disconnect(&mut self, from: &Output, to: &Input) -> Result<Edge, JsValue> {
        self.inner.disconnect(from.inner, to.inner);
        Ok(Edge {
            inner: raug::graph::Connection {
                source: from.inner.node,
                source_output: from.inner.index,
                target: to.inner.node,
                target_input: to.inner.index,
            },
        })
    }

    #[wasm_bindgen(js_name = "connectConstant")]
    pub fn connect_constant(&mut self, value: f32, to: &Input) -> Result<(), JsValue> {
        self.inner.connect_constant(value, to.inner);
        Ok(())
    }

    #[wasm_bindgen(js_name = "connectFloatParam")]
    pub fn connect_float_param(&mut self, value: f32, to: &Input) -> Result<FloatParam, JsValue> {
        let param = self.inner.connect_param(value, to.inner);
        Ok(FloatParam { inner: param })
    }

    #[wasm_bindgen(js_name = "connectBoolParam")]
    pub fn connect_bool_param(&mut self, value: bool, to: &Input) -> Result<BoolParam, JsValue> {
        let param = self.inner.connect_param(value, to.inner);
        Ok(BoolParam { inner: param })
    }

    #[wasm_bindgen(js_name = "audioOutput")]
    pub fn audio_output(&mut self) -> Result<Node, JsValue> {
        let node = self.inner.audio_output();
        Ok(Node { inner: node })
    }

    #[wasm_bindgen(js_name = "connectAudioInput")]
    pub fn connect_audio_input(&mut self, to: &Input) -> Result<(), JsValue> {
        self.inner.connect_audio_input(to.inner);
        Ok(())
    }

    #[wasm_bindgen(js_name = "connectAudioOutput")]
    pub fn connect_audio_output(&mut self, from: &Output) -> Result<(), JsValue> {
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
            #[allow(clippy::needless_range_loop)]
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
