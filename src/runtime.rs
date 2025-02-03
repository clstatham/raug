//! The audio graph processing runtime.

use std::{
    sync::{mpsc, Arc, Mutex},
    time::Duration,
};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use petgraph::prelude::*;
use rustc_hash::{FxBuildHasher, FxHashMap};

use crate::{
    debug_once,
    graph::{Graph, GraphRunError, GraphRunErrorType, NodeIndex},
    prelude::{Param, ProcessorInputs, SignalSpec},
    processor::{ProcessMode, ProcessorError, ProcessorOutputs},
    signal::{Float, MidiMessage, SignalBuffer},
};

/// Errors that can occur related to the runtime.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
#[error("Runtime error")]
pub enum RuntimeError {
    /// An error occurred while the stream was running.
    StreamError(#[from] cpal::StreamError),

    /// An error occurred while enumerating available audio devices.
    DevicesError(#[from] cpal::DevicesError),

    /// An error occurred while enumerating available hosts.
    Hound(#[from] hound::Error),

    /// The requested host is unavailable.
    HostUnavailable(#[from] cpal::HostUnavailable),

    /// The requested device is unavailable.
    #[error("Requested device is unavailable: {0:?}")]
    DeviceUnavailable(AudioDevice),

    /// An error occurred while retrieving the device name.
    DeviceNameError(#[from] cpal::DeviceNameError),

    /// An error occurred while retrieving the default output config.
    DefaultStreamConfigError(#[from] cpal::DefaultStreamConfigError),

    /// Output stream sample format is not supported.
    #[error("Unsupported sample format: {0}")]
    UnsupportedSampleFormat(cpal::SampleFormat),

    /// An error occurred while initializing MIDI input.
    MidirInitError(#[from] midir::InitError),

    /// The requested MIDI port is unavailable.
    #[error("Requested MIDI port is unavailable: {0:?}")]
    MidiPortUnavailable(MidiPort),

    /// An error occurred while connecting to a MIDI port.
    MidiConnectError(#[from] midir::ConnectError<midir::MidiInput>),

    /// An error occurred while running the audio graph.
    GraphRunError(#[from] GraphRunError),

    /// The runtime needs to reallocate buffers.
    NeedsAlloc,

    /// An error occurred while processing a node in the audio graph.
    ProcessorError(#[from] ProcessorError),

    /// The number of channels in the audio stream does not match the number of outputs in the graph.
    #[error("Channel mismatch: expected {0} channels, got {1}")]
    ChannelMismatch(usize, usize),
}

/// Result type for runtime operations.
pub type RuntimeResult<T> = Result<T, RuntimeError>;

/// The audio backend to use for audio I/O.
#[derive(Default, Debug, Clone)]
pub enum AudioBackend {
    /// Use the default audio backend.
    #[default]
    Default,
    #[cfg(all(target_os = "linux", feature = "jack"))]
    /// Use the JACK Audio Connection Kit audio backend.
    Jack,
    #[cfg(target_os = "linux")]
    /// Use the Advanced Linux Sound Architecture audio backend.
    Alsa,
    #[cfg(target_os = "windows")]
    /// Use the Windows Audio Session API audio backend.
    Wasapi,
}

/// An audio device to use for audio I/O.
#[derive(Default, Debug, Clone)]
pub enum AudioDevice {
    /// Use the default audio device.
    #[default]
    Default,
    /// Use the audio device at the given index.
    Index(usize),
    /// Use the audio device with the given substring in its name.
    Name(String),
}

/// A MIDI port to use for MIDI I/O.
#[derive(Default, Debug, Clone)]
pub enum MidiPort {
    /// Use the default MIDI port.
    #[default]
    Default,
    /// Use the MIDI port at the given index.
    Index(usize),
    /// Use the MIDI port with the given substring in its name.
    Name(String),
}

#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub(crate) struct NodeBuffers {
    input_spec: Vec<SignalSpec>,
    outputs: Vec<SignalBuffer>,
    output_spec: Vec<SignalSpec>,
}

impl NodeBuffers {
    fn resize(&mut self, block_size: usize) {
        for (spec, buffer) in self.output_spec.iter().zip(&mut self.outputs) {
            buffer.resize_with_hint(block_size, &spec.signal_type);
        }
    }
}

/// The audio graph processing runtime.
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Runtime {
    graph: Graph,
    buffer_cache: FxHashMap<NodeIndex, NodeBuffers>,
    sample_rate: Float,
    block_size: usize,
    max_block_size: usize,
}

impl Runtime {
    /// Creates a new runtime from the given graph.
    pub fn new(mut graph: Graph) -> Self {
        let mut buffer_cache =
            FxHashMap::with_capacity_and_hasher(graph.digraph().node_count(), FxBuildHasher);

        graph
            .visit(|graph, node_id| -> RuntimeResult<()> {
                let node = &graph.digraph()[node_id];
                let output_spec = node.output_spec();

                let mut outputs = Vec::with_capacity(output_spec.len());

                for spec in output_spec {
                    let buffer = SignalBuffer::new_of_type(&spec.signal_type, 0);
                    outputs.push(buffer);
                }

                buffer_cache.insert(
                    node_id,
                    NodeBuffers {
                        input_spec: node.input_spec().to_vec(),
                        output_spec: output_spec.to_vec(),
                        outputs,
                    },
                );

                Ok(())
            })
            .unwrap();

        Runtime {
            buffer_cache,
            graph,
            sample_rate: 0.0,
            block_size: 0,
            max_block_size: 0,
        }
    }

    /// Returns the current sample rate.
    #[inline]
    pub fn sample_rate(&self) -> Float {
        self.sample_rate
    }

    /// Returns the current block size.
    #[inline]
    pub fn block_size(&self) -> usize {
        self.block_size
    }

    /// Resets the runtime for the given sample rate and block size.
    ///
    /// This will reallocate buffers if necessary.
    #[inline]
    pub fn allocate_for_block_size(&mut self, sample_rate: Float, max_block_size: usize) {
        self.graph.reset_visitor();

        self.sample_rate = sample_rate;
        self.block_size = max_block_size;
        self.max_block_size = max_block_size;

        self.graph.allocate(sample_rate, max_block_size);
        self.graph.resize_buffers(sample_rate, max_block_size);

        for buffers in self.buffer_cache.values_mut() {
            buffers.resize(max_block_size);
        }
    }

    /// Resets the runtime for the given sample rate and block size.
    ///
    /// This is guaranteed to not allocate, assuming all processors are playing nicely. If it would need to allocate, it will return an error.
    #[inline]
    pub fn set_block_size(&mut self, block_size: usize) -> RuntimeResult<()> {
        if block_size > self.max_block_size {
            return Err(RuntimeError::NeedsAlloc);
        }

        if block_size == self.block_size {
            return Ok(());
        }

        self.block_size = block_size;

        self.graph.resize_buffers(self.sample_rate, block_size);

        for buffers in self.buffer_cache.values_mut() {
            buffers.resize(block_size);
        }

        Ok(())
    }

    /// Returns a reference to the audio graph.
    #[inline]
    pub fn graph(&self) -> &Graph {
        &self.graph
    }

    /// Returns a mutable reference to the audio graph.
    #[inline]
    pub fn graph_mut(&mut self) -> &mut Graph {
        &mut self.graph
    }

    /// Runs the audio graph for one block of samples.
    #[cfg_attr(feature = "profiling", inline(never))]
    pub fn process(&mut self) -> RuntimeResult<()> {
        for i in 0..self.graph.sccs().len() {
            if self.graph.sccs()[i].len() == 1 {
                let node_id = self.graph.sccs()[i][0];
                self.process_node(node_id, ProcessMode::Block)?;
            } else {
                let nodes = self.graph.sccs()[i].clone();
                for sample_index in 0..self.block_size {
                    for &node_id in &nodes {
                        self.process_node(node_id, ProcessMode::Sample(sample_index))?;
                    }
                }
            }
        }

        Ok(())
    }

    #[cfg_attr(feature = "profiling", inline(never))]
    fn process_node(&mut self, node_id: NodeIndex, mode: ProcessMode) -> RuntimeResult<()> {
        let num_inputs = self.buffer_cache[&node_id].input_spec.len();

        let mut inputs: smallvec::SmallVec<[_; 8]> = smallvec::smallvec![None; num_inputs];

        let mut buffers = self.buffer_cache.remove(&node_id).unwrap();

        for (source_id, edge) in self
            .graph
            .digraph()
            .edges_directed(node_id, Direction::Incoming)
            .map(|edge| (edge.source(), edge.weight()))
        {
            let source_buffers = self.buffer_cache.get(&source_id).unwrap();
            let buffer = &source_buffers.outputs[edge.source_output as usize];

            inputs[edge.target_input as usize] = Some(buffer);
        }

        let node = self.graph.digraph.node_weight_mut(node_id).unwrap();

        if inputs.spilled() {
            debug_once!(format!("{}_spilled", node_id.index()) => "Input array for {} ({}) spilled over to the heap (has {} inputs > 8)", node.name(), node_id.index(), num_inputs);
        }

        let result = node.process(
            ProcessorInputs::new(
                &buffers.input_spec,
                &inputs[..],
                &self.graph.assets,
                mode,
                self.sample_rate,
                self.block_size,
            ),
            ProcessorOutputs::new(&buffers.output_spec, &mut buffers.outputs, mode),
        );

        if let Err(err) = result {
            let node = self.graph.digraph.node_weight(node_id).unwrap();
            log::error!("Error processing node {}: {:?}", node.name(), err);
            let error = GraphRunError {
                node_index: node_id,
                node_processor: node.name().to_string(),
                signal_type: GraphRunErrorType::ProcessorError(err),
            };
            return Err(RuntimeError::GraphRunError(error));
        }

        drop(inputs);

        self.buffer_cache.insert(node_id, buffers);

        Ok(())
    }

    /// Returns a reference to the runtime's input buffer for the given input index.
    #[inline]
    pub fn get_input_mut(&mut self, input_index: usize) -> Option<&mut SignalBuffer> {
        self.buffer_cache
            .get_mut(self.graph.input_indices().get(input_index)?)
            .map(|buffers| &mut buffers.outputs[0])
    }

    /// Returns a reference to the runtime's output buffer for the given output index.
    #[inline]
    pub fn get_output(&self, output_index: usize) -> Option<&SignalBuffer> {
        self.buffer_cache
            .get(self.graph.output_indices().get(output_index)?)
            .map(|buffers| &buffers.outputs[0])
    }

    /// Returns a reference to the [`Param`] with the given name.
    #[inline]
    pub fn param_named(&self, name: &str) -> Option<&Param> {
        self.graph.param_named(name)
    }

    /// Runs the audio graph offline for the given duration and sample rate, returning the output buffers.
    pub fn run_offline(
        &mut self,
        duration: Duration,
        sample_rate: Float,
        block_size: usize,
    ) -> RuntimeResult<Box<[Box<[Float]>]>> {
        self.run_offline_inner(duration, sample_rate, block_size, false)
    }

    /// Runs the audio graph offline for the given duration and sample rate, returning the output buffers.
    ///
    /// This method will sleep for the duration of each block to simulate real-time processing.
    pub fn simulate(
        &mut self,
        duration: Duration,
        sample_rate: Float,
        block_size: usize,
    ) -> RuntimeResult<Box<[Box<[Float]>]>> {
        self.run_offline_inner(duration, sample_rate, block_size, true)
    }

    fn run_offline_inner(
        &mut self,
        duration: Duration,
        sample_rate: Float,
        block_size: usize,
        add_delay: bool,
    ) -> RuntimeResult<Box<[Box<[Float]>]>> {
        let secs = duration.as_secs_f64() as Float;
        let samples = (sample_rate * secs) as usize;

        self.allocate_for_block_size(sample_rate, block_size);

        let num_outputs: usize = self.graph.num_audio_outputs();

        let mut outputs: Box<[Box<[Float]>]> =
            vec![vec![0.0; samples].into_boxed_slice(); num_outputs].into_boxed_slice();

        let mut sample_count = 0;
        let mut last_block_size = 0;

        while sample_count < samples {
            let actual_block_size = (samples - sample_count).min(block_size);
            if actual_block_size != last_block_size {
                self.set_block_size(actual_block_size)?;
                last_block_size = actual_block_size;
            }
            self.process()?;

            for (i, output) in outputs.iter_mut().enumerate() {
                let buffer = self.get_output(i);
                let Some(SignalBuffer::Float(buffer)) = buffer else {
                    return Err(RuntimeError::ChannelMismatch(0, i));
                };

                for (j, &sample) in buffer[..actual_block_size].iter().enumerate() {
                    output[sample_count + j] = sample.unwrap_or_default();
                }
            }

            if add_delay {
                std::thread::sleep(Duration::from_secs_f64(
                    actual_block_size as f64 / sample_rate as f64,
                ));
            }

            sample_count += actual_block_size;
        }

        Ok(outputs)
    }

    /// Runs the audio graph offline for the given duration and sample rate, writing the output to a file.
    pub fn run_offline_to_file(
        &mut self,
        file_path: impl AsRef<std::path::Path>,
        duration: Duration,
        sample_rate: Float,
        block_size: usize,
    ) -> RuntimeResult<()> {
        let outputs = self.run_offline(duration, sample_rate, block_size)?;

        let num_channels = outputs.len();

        if num_channels == 0 {
            log::warn!("No output channels to write to file");
            return Ok(());
        }

        let num_samples = outputs[0].len();

        let mut samples = vec![0.0; num_samples * num_channels];

        for sample_index in 0..num_samples {
            for channel_index in 0..num_channels {
                let i = sample_index * num_channels + channel_index;
                samples[i] = outputs[channel_index][sample_index];
            }
        }

        let spec = hound::WavSpec {
            channels: num_channels as u16,
            sample_rate: sample_rate as u32,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };

        let mut writer = hound::WavWriter::create(file_path, spec)?;

        for sample in samples {
            writer.write_sample(sample as f32)?;
        }

        writer.finalize()?;

        Ok(())
    }

    /// Runs the audio graph in real-time for the given duration.
    pub fn run_for(
        &mut self,
        duration: Duration,
        backend: AudioBackend,
        device: AudioDevice,
        midi_port: Option<MidiPort>,
    ) -> RuntimeResult<()> {
        let handle = self.run(backend, device, midi_port)?;
        std::thread::sleep(duration);
        handle.stop();
        Ok(())
    }

    /// Starts running the audio graph in real-time. Returns a [`RuntimeHandle`] that can be used to stop the runtime.
    pub fn run(
        &mut self,
        backend: AudioBackend,
        device: AudioDevice,
        midi_port: Option<MidiPort>,
    ) -> RuntimeResult<RuntimeHandle> {
        let (kill_tx, kill_rx) = mpsc::channel();

        let host_id = match backend {
            AudioBackend::Default => cpal::default_host().id(),
            #[cfg(target_os = "linux")]
            AudioBackend::Alsa => cpal::available_hosts()
                .into_iter()
                .find(|h| *h == cpal::HostId::Alsa)
                .ok_or(RuntimeError::HostUnavailable(cpal::HostUnavailable))?,
            #[cfg(all(target_os = "linux", feature = "jack"))]
            AudioBackend::Jack => cpal::available_hosts()
                .into_iter()
                .find(|h| *h == cpal::HostId::Jack)
                .ok_or(RuntimeError::HostUnavailable(cpal::HostUnavailable))?,
            #[cfg(target_os = "windows")]
            AudioBackend::Wasapi => cpal::available_hosts()
                .into_iter()
                .find(|h| *h == cpal::HostId::Wasapi)
                .ok_or(RuntimeError::HostUnavailable(cpal::HostUnavailable))?,
        };
        let host = cpal::host_from_id(host_id)?;

        log::info!("Using host: {:?}", host.id());

        let cpal_device = match &device {
            AudioDevice::Default => host.default_output_device(),
            AudioDevice::Index(index) => host.output_devices().unwrap().nth(*index),
            AudioDevice::Name(name) => host
                .output_devices()
                .unwrap()
                .find(|d| d.name().unwrap().contains(name)),
        };

        let cpal_device = cpal_device.ok_or(RuntimeError::DeviceUnavailable(device))?;

        log::info!("Using device: {}", cpal_device.name()?);

        let config = cpal_device.default_output_config()?;

        let channels = config.channels();
        if self.graph.num_audio_outputs() != channels as usize {
            return Err(RuntimeError::ChannelMismatch(
                self.graph.num_audio_outputs(),
                channels as usize,
            ));
        }

        log::info!("Configuration: {:#?}", config);

        let audio_rate = config.sample_rate().0 as Float;

        let midi_connection = midir::MidiInput::new("raug midir input")?;

        let midi_port = if let Some(midi_port) = midi_port {
            let midi_port = match &midi_port {
                MidiPort::Default => midi_connection.ports().into_iter().next(),
                MidiPort::Index(index) => midi_connection.ports().into_iter().nth(*index),
                MidiPort::Name(name) => midi_connection
                    .ports()
                    .into_iter()
                    .find(|port| midi_connection.port_name(port).unwrap().contains(name)),
            }
            .ok_or(RuntimeError::MidiPortUnavailable(midi_port))?;

            log::info!(
                "Using MIDI port: {:?}",
                midi_connection
                    .port_name(&midi_port)
                    .as_ref()
                    .map(|s| s.as_str())
                    .unwrap_or("unknown")
            );

            Some(midi_port)
        } else {
            None
        };

        self.allocate_for_block_size(audio_rate, audio_rate as usize / 10);

        let audio_runtime = self.clone();
        let midi_runtime = self.clone();

        let midi_in = if let Some(midi_port) = midi_port {
            let midi_in = midi_connection.connect(
                &midi_port,
                "raug midir input",
                move |_stamp, message, _data| {
                    log::debug!("MIDI message: {:2x?}", message);

                    for (_name, param) in midi_runtime.graph().midi_input_iter() {
                        param.send(MidiMessage::new([message[0], message[1], message[2]]));
                    }
                },
                (),
            )?;

            Some(midi_in)
        } else {
            None
        };

        let handle = RuntimeHandle {
            kill_tx,
            midi_in: Arc::new(Mutex::new(midi_in)),
        };

        std::thread::spawn(move || -> RuntimeResult<()> {
            let stream = match config.sample_format() {
                cpal::SampleFormat::I8 => {
                    audio_runtime.run_inner::<i8>(&cpal_device, &config.config())?
                }
                cpal::SampleFormat::I16 => {
                    audio_runtime.run_inner::<i16>(&cpal_device, &config.config())?
                }
                cpal::SampleFormat::I32 => {
                    audio_runtime.run_inner::<i32>(&cpal_device, &config.config())?
                }
                cpal::SampleFormat::I64 => {
                    audio_runtime.run_inner::<i64>(&cpal_device, &config.config())?
                }
                cpal::SampleFormat::U8 => {
                    audio_runtime.run_inner::<u8>(&cpal_device, &config.config())?
                }
                cpal::SampleFormat::U16 => {
                    audio_runtime.run_inner::<u16>(&cpal_device, &config.config())?
                }
                cpal::SampleFormat::U32 => {
                    audio_runtime.run_inner::<u32>(&cpal_device, &config.config())?
                }
                cpal::SampleFormat::U64 => {
                    audio_runtime.run_inner::<u64>(&cpal_device, &config.config())?
                }
                cpal::SampleFormat::F32 => {
                    audio_runtime.run_inner::<f32>(&cpal_device, &config.config())?
                }
                cpal::SampleFormat::F64 => {
                    audio_runtime.run_inner::<f64>(&cpal_device, &config.config())?
                }

                sample_format => {
                    return Err(RuntimeError::UnsupportedSampleFormat(sample_format));
                }
            };

            loop {
                if kill_rx.try_recv().is_ok() {
                    drop(stream);
                    break;
                }

                std::thread::yield_now();
            }

            Ok(())
        });

        Ok(handle)
    }

    fn run_inner<T>(
        mut self,
        device: &cpal::Device,
        config: &cpal::StreamConfig,
    ) -> RuntimeResult<cpal::Stream>
    where
        T: cpal::SizedSample + cpal::FromSample<Float>,
    {
        let channels = config.channels as usize;

        let mut last_block_size = 0;
        let stream = device
            .build_output_stream(
                config,
                move |data: &mut [T], _info: &cpal::OutputCallbackInfo| {
                    let block_size = data.len() / channels;
                    if block_size != last_block_size {
                        self.set_block_size(block_size).unwrap();
                        last_block_size = block_size;
                    }

                    self.process().unwrap();

                    for (frame_idx, frame) in data.chunks_mut(channels).enumerate() {
                        for (channel_idx, sample) in frame.iter_mut().enumerate() {
                            let buffer = self.get_output(channel_idx);
                            let Some(SignalBuffer::Float(buffer)) = buffer else {
                                panic!("output {channel_idx} signal type mismatch");
                            };
                            let value = buffer[frame_idx].unwrap_or_default();
                            *sample = T::from_sample(value);
                        }
                    }
                },
                |err| eprintln!("an error occurred on output: {}", err),
                None,
            )
            .unwrap();

        stream.play().unwrap();

        Ok(stream)
    }
}

/// A handle to the runtime that can be used to stop it.
#[must_use = "The runtime handle must be kept alive for the runtime to continue running"]
#[derive(Clone)]
pub struct RuntimeHandle {
    midi_in: Arc<Mutex<Option<midir::MidiInputConnection<()>>>>,
    kill_tx: mpsc::Sender<()>,
}

impl RuntimeHandle {
    /// Stops the runtime. This will close the audio stream and MIDI input.
    pub fn stop(&self) {
        self.kill_tx.send(()).ok();
        if let Ok(mut midi_in) = self.midi_in.lock() {
            if let Some(midi_in) = midi_in.take() {
                midi_in.close();
            }
        }
    }
}

impl Drop for RuntimeHandle {
    fn drop(&mut self) {
        self.stop();
    }
}
