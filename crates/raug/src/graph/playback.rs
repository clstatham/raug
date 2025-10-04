//! The audio graph processing runtime.

use std::{
    fs::File,
    io::BufWriter,
    path::Path,
    str::FromStr,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    thread::JoinHandle,
    time::Duration,
};

use atomic_time::AtomicDuration;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam_channel::{Receiver, RecvError, SendError, Sender, TryRecvError, TrySendError};

use crate::{
    graph::{
        Graph, GraphConstructionError, Node,
        node::{Input, Output},
    },
    prelude::{ProcessNodeError, Processor, ProcessorNode, RaugNodeIndexExt},
};

/// The type of error that occurred while running a graph.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
#[error("Graph run error")]
pub enum GraphRunError {
    #[error("Unknown audio backend: {0}")]
    UnknownBackend(String),

    /// An error occurred while processing the node.
    ProcessorNodeError(#[from] ProcessNodeError),

    /// An error occurred while the stream was running.
    StreamError(#[from] cpal::StreamError),

    /// An error occurred while playing the stream.
    StreamPlayError(#[from] cpal::PlayStreamError),

    /// An error occurred while pausing the stream.
    StreamPauseError(#[from] cpal::PauseStreamError),

    /// An error occurred while enumerating available audio devices.
    DevicesError(#[from] cpal::DevicesError),

    /// An error occurred reading or writing the WAV file.
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

    /// An error occurred while modifying the graph.
    #[error("Graph modification error: {0}")]
    GraphModificationError(#[from] GraphConstructionError),

    /// An error occurred while sending data to the audio stream.
    #[error("Stream send error")]
    StreamSendError,

    /// An error occurred while receiving data from the audio stream.
    #[error("Stream receive error")]
    StreamReceiveError,

    /// Audio stream is not spawned.
    #[error("Audio stream not spawned")]
    StreamNotSpawned,
}

/// A result type for graph run operations.
pub type GraphRunResult<T> = Result<T, GraphRunError>;

impl Graph {
    pub fn play(mut self, mut output_stream: impl AudioOut) -> GraphRunResult<RunningGraph> {
        let status = Arc::new(GraphStatus::new(self.sample_rate as usize, self.block_size));
        let status_clone = status.clone();

        let (kill_tx, kill_rx) = crossbeam_channel::bounded(1);
        let (request_tx, request_rx) = crossbeam_channel::unbounded();
        let (response_tx, response_rx) = crossbeam_channel::unbounded();
        let handle = std::thread::spawn(move || -> GraphRunResult<Graph> {
            loop {
                if kill_rx.try_recv().is_ok() {
                    return Ok(self);
                }

                while let Ok(request) = request_rx.try_recv() {
                    match request {
                        GraphRequest::AddNode(node) => {
                            let node = self.graph.add_node(node);
                            response_tx
                                .send(GraphResponse::NodeAdded(Node(node)))
                                .map_err(|_| GraphRunError::StreamSendError)?;
                        }
                        GraphRequest::RemoveNode(node) => {
                            self.disconnect_all(node);
                            response_tx
                                .send(GraphResponse::NodeRemoved(node))
                                .map_err(|_| GraphRunError::StreamSendError)?;
                        }
                        GraphRequest::ConnectU32U32 {
                            source,
                            source_output,
                            target,
                            target_input,
                        } => {
                            if source_output < self.graph[source.0].num_outputs() as u32
                                && target_input < self.graph[target.0].num_inputs() as u32
                            {
                                self.connect(
                                    source.output(source_output),
                                    target.input(target_input),
                                );
                                response_tx
                                    .send(GraphResponse::Connected)
                                    .map_err(|_| GraphRunError::StreamSendError)?;
                            } else {
                                return Err(GraphRunError::GraphModificationError(
                                    GraphConstructionError::InputIndexOutOfBounds {
                                        index: target_input as usize,
                                        num_inputs: self.graph[target.0].num_inputs(),
                                    },
                                ));
                            }
                        }
                    }
                }

                while output_stream.output_samples_needed() > 0 {
                    if kill_rx.try_recv().is_ok() {
                        return Ok(self);
                    }

                    let block_size = output_stream.block_size();
                    if block_size > self.max_block_size {
                        log::debug!("Reallocating graph buffers to {} samples", block_size);
                        self.allocate(output_stream.sample_rate(), block_size);
                        status_clone.block_size.store(block_size, Ordering::Relaxed);
                    } else if block_size != self.block_size {
                        log::debug!("Resizing graph buffers to {} samples", block_size);
                        self.resize_buffers(output_stream.sample_rate(), block_size);
                        status_clone.block_size.store(block_size, Ordering::Relaxed);
                    }

                    status_clone
                        .sample_rate
                        .store(output_stream.sample_rate() as usize, Ordering::Relaxed);

                    self.process()?;

                    let mut delta = 0;
                    for sample_idx in 0..self.block_size {
                        for channel_idx in 0..output_stream.output_channels() {
                            let Some(buffer) = self.get_output(channel_idx) else {
                                continue;
                            };

                            delta += output_stream.write(&[buffer[sample_idx]])?;
                        }
                    }

                    status_clone
                        .samples_written
                        .fetch_add(delta, Ordering::Relaxed);

                    let duration_secs = delta as f32
                        / output_stream.output_channels() as f32
                        / output_stream.sample_rate();
                    status_clone
                        .duration_written
                        .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |dur| {
                            Some(dur + Duration::from_secs_f32(duration_secs))
                        })
                        .unwrap();
                }
            }
        });

        Ok(RunningGraph {
            handle,
            kill_tx,
            request_tx,
            response_rx,
            status,
        })
    }

    pub fn play_for(
        self,
        output_stream: impl AudioOut,
        duration: Duration,
    ) -> GraphRunResult<Graph> {
        let handle = self.play(output_stream)?;
        let graph = handle.run_for(duration)?;
        Ok(graph)
    }
}

pub struct GraphStatus {
    samples_written: AtomicUsize,
    duration_written: AtomicDuration,
    sample_rate: AtomicUsize,
    block_size: AtomicUsize,
}

impl GraphStatus {
    fn new(sample_rate: usize, block_size: usize) -> Self {
        Self {
            samples_written: AtomicUsize::new(0),
            duration_written: AtomicDuration::new(Duration::ZERO),
            sample_rate: AtomicUsize::new(sample_rate),
            block_size: AtomicUsize::new(block_size),
        }
    }
}

pub enum GraphRequest {
    AddNode(ProcessorNode),
    RemoveNode(Node),
    ConnectU32U32 {
        source: Node,
        source_output: u32,
        target: Node,
        target_input: u32,
    },
}

#[derive(Debug)]
pub enum GraphResponse {
    NodeAdded(Node),
    NodeRemoved(Node),
    Connected,
}

pub struct RunningGraph {
    handle: JoinHandle<GraphRunResult<Graph>>,
    kill_tx: Sender<()>,
    request_tx: Sender<GraphRequest>,
    response_rx: Receiver<GraphResponse>,
    status: Arc<GraphStatus>,
}

impl RunningGraph {
    pub fn stop(self) -> GraphRunResult<Graph> {
        self.kill_tx.send(()).unwrap();
        self.handle.join().unwrap()
    }

    pub fn samples_written(&self) -> usize {
        self.status.samples_written.load(Ordering::Relaxed)
    }

    pub fn duration_written(&self) -> Duration {
        self.status.duration_written.load(Ordering::Relaxed)
    }

    pub fn sample_rate(&self) -> usize {
        self.status.sample_rate.load(Ordering::Relaxed)
    }

    pub fn block_size(&self) -> usize {
        self.status.block_size.load(Ordering::Relaxed)
    }

    pub fn run_for(self, duration: Duration) -> GraphRunResult<Graph> {
        while self.duration_written() < duration {
            std::thread::sleep(Duration::from_millis(10));
        }
        self.stop()
    }

    pub fn node(&self, node: impl Processor) -> GraphRunResult<Node> {
        let mut node = ProcessorNode::new(node);
        node.allocate(self.sample_rate() as f32, self.block_size());
        node.resize_buffers(self.sample_rate() as f32, self.block_size());
        match self.request_tx.send(GraphRequest::AddNode(node)) {
            Ok(()) => {}
            Err(_) => return Err(GraphRunError::StreamNotSpawned),
        };

        match self.response_rx.recv() {
            Ok(GraphResponse::NodeAdded(node)) => Ok(node),
            Ok(_) => Err(GraphRunError::StreamReceiveError),
            Err(_) => Err(GraphRunError::StreamReceiveError),
        }
    }

    pub fn remove_node(&self, node: Node) -> GraphRunResult<Node> {
        match self.request_tx.send(GraphRequest::RemoveNode(node)) {
            Ok(()) => {}
            Err(_) => return Err(GraphRunError::StreamNotSpawned),
        };

        match self.response_rx.recv() {
            Ok(GraphResponse::NodeRemoved(node)) => Ok(node),
            Ok(_) => Err(GraphRunError::StreamReceiveError),
            Err(_) => Err(GraphRunError::StreamReceiveError),
        }
    }

    pub fn connect(&self, source: Output<u32>, target: Input<u32>) -> GraphRunResult<()> {
        match self.request_tx.send(GraphRequest::ConnectU32U32 {
            source: source.node.into(),
            source_output: source.index,
            target: target.node.into(),
            target_input: target.index,
        }) {
            Ok(()) => {}
            Err(_) => return Err(GraphRunError::StreamNotSpawned),
        };

        match self.response_rx.recv() {
            Ok(GraphResponse::Connected) => Ok(()),
            Ok(_) => Err(GraphRunError::StreamReceiveError),
            Err(_) => Err(GraphRunError::StreamReceiveError),
        }
    }
}

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

impl FromStr for AudioBackend {
    type Err = GraphRunError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().trim() {
            "default" => Ok(Self::Default),
            #[cfg(all(target_os = "linux", feature = "jack"))]
            "jack" => Ok(Self::Jack),
            #[cfg(target_os = "linux")]
            "alsa" => Ok(Self::Alsa),
            #[cfg(target_os = "windows")]
            "wasapi" => Ok(Self::Wasapi),
            _ => Err(GraphRunError::UnknownBackend(s.to_string())),
        }
    }
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

impl FromStr for AudioDevice {
    type Err = GraphRunError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.to_ascii_lowercase().trim() == "default" {
            Ok(Self::Default)
        } else if let Ok(i) = s.trim().parse() {
            Ok(Self::Index(i))
        } else {
            Ok(Self::Name(s.to_string()))
        }
    }
}

/// Utility struct for creating channels for communication between threads.
#[derive(Debug)]
pub struct Channels<T> {
    tx: Sender<T>,
    rx: Receiver<T>,
}

impl<T> Default for Channels<T> {
    fn default() -> Self {
        Self::unbounded()
    }
}

impl<T> Clone for Channels<T> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
            rx: self.rx.clone(),
        }
    }
}

impl<T> Channels<T> {
    /// Creates a new `Channels` instance with unbounded channels.
    pub fn unbounded() -> Self {
        let (tx, rx) = crossbeam_channel::unbounded();
        Self { tx, rx }
    }

    /// Creates a new `Channels` instance with bounded channels of the given capacity.
    pub fn bounded(cap: usize) -> Self {
        let (tx, rx) = crossbeam_channel::bounded(cap);
        Self { tx, rx }
    }

    /// Returns the sender end of the channel.
    pub fn tx(&self) -> &Sender<T> {
        &self.tx
    }

    /// Returns the receiver end of the channel.
    pub fn rx(&self) -> &Receiver<T> {
        &self.rx
    }

    /// Tries to send data through the channel without blocking.
    pub fn try_send(&self, data: T) -> Result<(), TrySendError<T>> {
        self.tx.try_send(data)
    }

    /// Returns `true` if the channel is full, `false` otherwise.
    pub fn is_full(&self) -> bool {
        self.tx.is_full()
    }

    /// Returns `true` if the channel is empty, `false` otherwise.
    pub fn is_empty(&self) -> bool {
        self.rx.is_empty()
    }

    /// Sends data through the channel, blocking until it can be sent.
    pub fn send_blocking(&self, data: T) -> Result<(), SendError<T>> {
        self.tx.send(data)
    }

    /// Tries to send all data from the iterator through the channel without blocking.
    pub fn send_all(&self, data: impl Iterator<Item = T>) -> Result<(), TrySendError<T>> {
        for item in data {
            self.try_send(item)?;
        }
        Ok(())
    }

    /// Sends all data from the iterator through the channel, blocking until it can be sent.
    pub fn send_all_blocking(&self, data: impl Iterator<Item = T>) -> Result<(), SendError<T>> {
        for item in data {
            self.send_blocking(item)?;
        }
        Ok(())
    }

    /// Tries to receive data from the channel without blocking.
    pub fn try_recv(&self) -> Result<Option<T>, TryRecvError> {
        match self.rx.try_recv() {
            Ok(data) => Ok(Some(data)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => Err(TryRecvError::Disconnected),
        }
    }

    /// Receives data from the channel, blocking until data is available.
    pub fn recv_blocking(&self) -> Result<T, RecvError> {
        self.rx.recv()
    }
}

/// An audio stream interface for outputting audio data.
pub trait AudioOut: Send + Sync + 'static {
    /// Returns the sample rate of the audio stream.
    fn sample_rate(&self) -> f32;
    /// Returns the block size of the audio stream.
    fn block_size(&self) -> usize;
    /// Returns the number of output channels of the audio stream.
    fn output_channels(&self) -> usize;

    /// Returns the number of output samples the stream needs from the graph.
    /// Negative values indicate that the stream has enough data already.
    fn output_samples_needed(&self) -> isize;

    /// Writes the given samples to the stream. On success, returns the number of samples written.
    fn write(&mut self, samps: &[f32]) -> GraphRunResult<usize>;
}

impl AudioOut for Box<dyn AudioOut> {
    fn sample_rate(&self) -> f32 {
        self.as_ref().sample_rate()
    }

    fn block_size(&self) -> usize {
        self.as_ref().block_size()
    }

    fn output_channels(&self) -> usize {
        self.as_ref().output_channels()
    }

    fn output_samples_needed(&self) -> isize {
        self.as_ref().output_samples_needed()
    }

    fn write(&mut self, samps: &[f32]) -> GraphRunResult<usize> {
        self.as_mut().write(samps)
    }
}

pub struct ParallelOut<A: AudioOut, B: AudioOut> {
    a: A,
    b: B,
}

impl<A: AudioOut, B: AudioOut> ParallelOut<A, B> {
    pub fn new(a: A, b: B) -> Self {
        Self { a, b }
    }
}

impl<A: AudioOut, B: AudioOut> AudioOut for ParallelOut<A, B> {
    fn sample_rate(&self) -> f32 {
        self.a.sample_rate()
    }

    fn block_size(&self) -> usize {
        self.a.block_size()
    }

    fn output_channels(&self) -> usize {
        self.a.output_channels()
    }

    fn output_samples_needed(&self) -> isize {
        self.a
            .output_samples_needed()
            .min(self.b.output_samples_needed())
    }

    fn write(&mut self, samps: &[f32]) -> GraphRunResult<usize> {
        for &samp in samps {
            self.a.write(&[samp])?;
            self.b.write(&[samp])?;
        }
        Ok(samps.len())
    }
}

/// An [`AudioOut`] implementation that discards all audio data, while still behaving like a real audio output.
/// Useful for testing and benchmarking.
pub struct NullOut {
    sample_rate: f32,
    block_size: usize,
    output_channels: usize,
}

impl NullOut {
    pub fn new(sample_rate: f32, block_size: usize, output_channels: usize) -> Self {
        Self {
            sample_rate,
            block_size,
            output_channels,
        }
    }
}

impl AudioOut for NullOut {
    fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    fn block_size(&self) -> usize {
        self.block_size
    }

    fn output_channels(&self) -> usize {
        self.output_channels
    }

    fn output_samples_needed(&self) -> isize {
        self.block_size as isize * self.output_channels as isize
    }

    fn write(&mut self, samps: &[f32]) -> GraphRunResult<usize> {
        Ok(samps.len())
    }
}

/// An [`AudioOut`] implementation that writes audio data to a WAV file.
pub struct WavFileOut {
    file: hound::WavWriter<BufWriter<File>>,
    sample_rate: f32,
    block_size: usize,
    output_channels: usize,
    samples_written: usize,
    max_samples: Option<usize>,
}

impl WavFileOut {
    /// Creates a new `WavFileOutStream` with the given parameters.
    pub fn new(
        filename: impl AsRef<Path>,
        sample_rate: f32,
        block_size: usize,
        output_channels: usize,
        max_duration: Option<Duration>,
    ) -> Self {
        let spec = hound::WavSpec {
            channels: output_channels as u16,
            sample_rate: sample_rate as u32,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        let file = hound::WavWriter::create(filename, spec).unwrap();

        let max_samples = if let Some(max_duration) = max_duration {
            let mut samples = (max_duration.as_secs_f32() * sample_rate) as usize;
            if !samples.is_multiple_of(block_size) {
                samples += block_size - (samples % block_size);
            }
            Some(samples * output_channels)
        } else {
            None
        };

        Self {
            file,
            sample_rate,
            block_size,
            output_channels,
            samples_written: 0,
            max_samples,
        }
    }

    /// Finalizes the WAV file, writing any remaining data and closing the file.
    pub fn finalize(self) -> hound::Result<()> {
        self.file.finalize()?;

        Ok(())
    }
}

impl AudioOut for WavFileOut {
    fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    fn block_size(&self) -> usize {
        self.block_size
    }

    fn output_channels(&self) -> usize {
        self.output_channels
    }

    fn output_samples_needed(&self) -> isize {
        if let Some(max_samples) = self.max_samples {
            max_samples as isize - self.samples_written as isize
        } else {
            self.block_size as isize * self.output_channels as isize
        }
    }

    fn write(&mut self, samps: &[f32]) -> GraphRunResult<usize> {
        let mut written = 0;
        for &samp in samps {
            self.file.write_sample(samp)?;
            written += 1;
        }
        Ok(written)
    }
}

/// An [`AudioOut`] implementation using the [`cpal`] crate for audio I/O with the system's sound card.
pub struct CpalOut {
    config: cpal::SupportedStreamConfig,
    samples: Channels<f32>,
    kill_tx: Sender<()>,
    block_size: Arc<AtomicUsize>,
}

impl CpalOut {
    /// Spawns a [`cpal`] stream on the given backend and device.
    pub fn spawn(backend: &AudioBackend, output_device: &AudioDevice) -> Self {
        let host = match backend {
            AudioBackend::Default => cpal::default_host(),
            #[cfg(all(target_os = "linux", feature = "jack"))]
            AudioBackend::Jack => cpal::host_from_id(cpal::HostId::Jack).unwrap(),
            #[cfg(target_os = "linux")]
            AudioBackend::Alsa => cpal::host_from_id(cpal::HostId::Alsa).unwrap(),
            #[cfg(target_os = "windows")]
            AudioBackend::Wasapi => cpal::host_from_id(cpal::HostId::Wasapi).unwrap(),
        };

        let output_device = match output_device {
            AudioDevice::Default => host.default_output_device().unwrap(),
            AudioDevice::Index(index) => host.output_devices().unwrap().nth(*index).unwrap(),
            AudioDevice::Name(name) => host
                .output_devices()
                .unwrap()
                .find(|d| d.name().is_ok_and(|n| n.contains(name.as_str())))
                .unwrap(),
        };
        let output_device = Arc::new(output_device);

        let output_config = output_device.default_output_config().unwrap();

        let block_size = Arc::new(AtomicUsize::new(512)); // initialize with a default block size
        let block_size_clone = block_size.clone();

        let samples = Channels::unbounded();
        let samples_clone = samples.clone();

        let (kill_tx, kill_rx) = crossbeam_channel::bounded(1);

        match output_config.sample_format() {
            cpal::SampleFormat::F32 => build_output_stream::<f32>(
                samples_clone,
                output_device.clone(),
                output_config.config(),
                block_size,
                kill_rx,
            ),
            cpal::SampleFormat::F64 => build_output_stream::<f64>(
                samples_clone,
                output_device.clone(),
                output_config.config(),
                block_size,
                kill_rx,
            ),
            cpal::SampleFormat::I8 => build_output_stream::<i8>(
                samples_clone,
                output_device.clone(),
                output_config.config(),
                block_size,
                kill_rx,
            ),
            cpal::SampleFormat::I16 => build_output_stream::<i16>(
                samples_clone,
                output_device.clone(),
                output_config.config(),
                block_size,
                kill_rx,
            ),
            cpal::SampleFormat::I32 => build_output_stream::<i32>(
                samples_clone,
                output_device.clone(),
                output_config.config(),
                block_size,
                kill_rx,
            ),
            cpal::SampleFormat::I64 => build_output_stream::<i64>(
                samples_clone,
                output_device.clone(),
                output_config.config(),
                block_size,
                kill_rx,
            ),
            cpal::SampleFormat::U8 => build_output_stream::<u8>(
                samples_clone,
                output_device.clone(),
                output_config.config(),
                block_size,
                kill_rx,
            ),
            cpal::SampleFormat::U16 => build_output_stream::<u16>(
                samples_clone,
                output_device.clone(),
                output_config.config(),
                block_size,
                kill_rx,
            ),
            cpal::SampleFormat::U32 => build_output_stream::<u32>(
                samples_clone,
                output_device.clone(),
                output_config.config(),
                block_size,
                kill_rx,
            ),
            cpal::SampleFormat::U64 => build_output_stream::<u64>(
                samples_clone,
                output_device.clone(),
                output_config.config(),
                block_size,
                kill_rx,
            ),

            _ => panic!("Unsupported sample format for output stream"),
        };

        Self {
            config: output_config,
            samples,
            block_size: block_size_clone,
            kill_tx,
        }
    }

    pub fn record_to_wav(self, filename: impl AsRef<Path>) -> ParallelOut<Self, WavFileOut> {
        let wav = WavFileOut::new(
            filename,
            self.sample_rate(),
            self.block_size(),
            self.output_channels(),
            None,
        );
        ParallelOut::new(self, wav)
    }
}

impl Drop for CpalOut {
    fn drop(&mut self) {
        let _ = self.kill_tx.send(());
    }
}

fn build_output_stream<T: cpal::SizedSample + cpal::FromSample<f32> + Send + 'static>(
    samples: Channels<f32>,
    output_device: Arc<cpal::Device>,
    config: cpal::StreamConfig,
    block_size: Arc<AtomicUsize>,
    kill_rx: Receiver<()>,
) -> JoinHandle<()> {
    let channels = config.channels as usize;
    std::thread::spawn(move || {
        let stream = output_device
            .build_output_stream(
                &config,
                move |data: &mut [T], _info| {
                    let data_len = data.len();
                    let new_block_size = data_len / channels;
                    let old_block_size = block_size.load(Ordering::Relaxed);
                    if new_block_size != old_block_size {
                        log::debug!(
                            "Changing block size from {} to {}",
                            old_block_size,
                            new_block_size
                        );
                        block_size.store(new_block_size, Ordering::Relaxed);
                    }

                    for out_samp in data.iter_mut() {
                        if let Ok(in_samp) = samples.recv_blocking() {
                            *out_samp = T::from_sample(in_samp);
                        } else {
                            log::error!("samples.recv_blocking() returned Err");
                            *out_samp = T::from_sample(0.0f32);
                        }
                    }
                },
                |err| {
                    log::error!("Output stream error: {}", err);
                },
                None,
            )
            .expect("Failed to build output stream");

        stream.play().unwrap();
        kill_rx.recv().unwrap();
    })
}

impl AudioOut for CpalOut {
    fn sample_rate(&self) -> f32 {
        self.config.sample_rate().0 as f32
    }

    fn block_size(&self) -> usize {
        self.block_size.load(Ordering::Relaxed)
    }

    fn output_channels(&self) -> usize {
        self.config.channels() as usize
    }

    fn output_samples_needed(&self) -> isize {
        let in_channel = self.samples.rx.len();
        self.block_size() as isize - in_channel as isize
    }

    fn write(&mut self, samps: &[f32]) -> GraphRunResult<usize> {
        let mut written = 0;
        for &samp in samps {
            self.samples.send_blocking(samp).unwrap();
            written += 1;
        }
        Ok(written)
    }
}
