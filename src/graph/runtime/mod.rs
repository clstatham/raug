//! The audio graph processing runtime.

use std::{
    fs::File,
    io::BufWriter,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use cpal::{
    Sample,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use crossbeam_channel::{Receiver, RecvError, SendError, Sender, TryRecvError, TrySendError};

use crate::signal::{Signal, buffer::SignalBuffer};

use super::{Graph, GraphRunError, GraphRunResult};

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
    pub fn unbounded() -> Self {
        let (tx, rx) = crossbeam_channel::unbounded();
        Self { tx, rx }
    }

    pub fn bounded(cap: usize) -> Self {
        let (tx, rx) = crossbeam_channel::bounded(cap);
        Self { tx, rx }
    }

    pub fn tx(&self) -> &Sender<T> {
        &self.tx
    }

    pub fn rx(&self) -> &Receiver<T> {
        &self.rx
    }

    pub fn try_send(&self, data: T) -> Result<(), TrySendError<T>> {
        self.tx.try_send(data)
    }

    pub fn is_full(&self) -> bool {
        self.tx.is_full()
    }

    pub fn is_empty(&self) -> bool {
        self.rx.is_empty()
    }

    pub fn send_blocking(&self, data: T) -> Result<(), SendError<T>> {
        self.tx.send(data)
    }

    pub fn send_all(&self, data: impl Iterator<Item = T>) -> Result<(), TrySendError<T>> {
        for item in data {
            self.try_send(item)?;
        }
        Ok(())
    }

    pub fn send_all_blocking(&self, data: impl Iterator<Item = T>) -> Result<(), SendError<T>> {
        for item in data {
            self.send_blocking(item)?;
        }
        Ok(())
    }

    pub fn try_recv(&self) -> Result<Option<T>, TryRecvError> {
        match self.rx.try_recv() {
            Ok(data) => Ok(Some(data)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => Err(TryRecvError::Disconnected),
        }
    }

    pub fn recv_blocking(&self) -> Result<T, RecvError> {
        self.rx.recv()
    }
}

pub trait AudioStream: Send + 'static {
    fn sample_rate(&self) -> f32;
    fn block_size(&self) -> usize;
    fn input_channels(&self) -> usize;
    fn output_channels(&self) -> usize;

    fn spawn(&mut self, graph: &Graph) -> GraphRunResult<()>;
    fn play(&mut self) -> GraphRunResult<()>;
    fn pause(&mut self) -> GraphRunResult<()>;
    fn stop(&mut self) -> GraphRunResult<()>;
}

pub struct WavFileOutStream {
    file: hound::WavWriter<BufWriter<File>>,
    sample_rate: f32,
    block_size: usize,
    input_channels: usize,
    output_channels: usize,
    written_samples: usize,
    max_samples: Option<usize>,
}

impl WavFileOutStream {
    pub fn new(
        file_path: &str,
        sample_rate: f32,
        block_size: usize,
        input_channels: usize,
        output_channels: usize,
        max_duration: Option<Duration>,
    ) -> Self {
        let spec = hound::WavSpec {
            channels: output_channels as u16,
            sample_rate: sample_rate as u32,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        let file = hound::WavWriter::create(file_path, spec).unwrap();

        let max_samples = if let Some(max_duration) = max_duration {
            let mut samples = (max_duration.as_secs_f64() as f32 * sample_rate) as usize;
            if samples % block_size != 0 {
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
            input_channels,
            output_channels,
            written_samples: 0,
            max_samples,
        }
    }

    pub fn finalize(self) -> hound::Result<()> {
        self.file.finalize()
    }
}

impl AudioStream for WavFileOutStream {
    fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    fn block_size(&self) -> usize {
        self.block_size
    }

    fn input_channels(&self) -> usize {
        self.input_channels
    }

    fn output_channels(&self) -> usize {
        self.output_channels
    }

    fn spawn(&mut self, graph: &Graph) -> GraphRunResult<()> {
        graph.allocate(self.sample_rate, self.block_size);
        graph.resize_buffers(self.sample_rate, self.block_size);

        let mut samples = vec![0.0; self.block_size * self.output_channels];
        loop {
            graph.with_inner(|graph| {
                graph.process().unwrap();
                for i in 0..self.output_channels {
                    let buffer = graph.get_output(i);
                    let Some(SignalBuffer::Float(buffer)) = buffer else {
                        continue;
                    };
                    for (j, &sample) in buffer[..self.block_size].iter().enumerate() {
                        let sample = f32::from_repr(sample.unwrap_or_default());
                        samples[j * self.output_channels + i] = sample;
                    }
                }
            });

            if let Some(max_samples) = self.max_samples {
                if self.written_samples >= max_samples {
                    break;
                }
            }

            for sample in &samples {
                self.file.write_sample(*sample).unwrap();
            }
            self.written_samples += self.block_size * self.output_channels;
        }

        Ok(())
    }

    fn play(&mut self) -> GraphRunResult<()> {
        Ok(())
    }

    fn pause(&mut self) -> GraphRunResult<()> {
        Ok(())
    }

    fn stop(&mut self) -> GraphRunResult<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
enum StreamOps {
    Play,
    Pause,
    Stop,
}

struct StreamThread {
    ops: Channels<StreamOps>,
}

impl StreamThread {
    fn spawn(build_stream: impl FnOnce() -> cpal::Stream + Send + Sync + 'static) -> Self {
        let ops = Channels::unbounded();
        let ops_clone = ops.clone();
        std::thread::spawn(move || {
            let stream = build_stream();
            while let Ok(op) = ops_clone.recv_blocking() {
                match op {
                    StreamOps::Play => {
                        stream.play().unwrap();
                    }
                    StreamOps::Pause => {
                        stream.pause().unwrap();
                    }
                    StreamOps::Stop => {
                        stream.pause().unwrap();
                        break;
                    }
                }
            }
        });
        Self { ops }
    }

    fn play(&self) -> GraphRunResult<()> {
        self.ops
            .try_send(StreamOps::Play)
            .map_err(|_| GraphRunError::StreamSendError)?;
        Ok(())
    }

    fn pause(&self) -> GraphRunResult<()> {
        self.ops
            .try_send(StreamOps::Pause)
            .map_err(|_| GraphRunError::StreamSendError)?;
        Ok(())
    }

    fn stop(&self) -> GraphRunResult<()> {
        self.ops
            .try_send(StreamOps::Stop)
            .map_err(|_| GraphRunError::StreamSendError)?;
        Ok(())
    }
}

pub struct CpalStream {
    output_device: Arc<cpal::Device>,
    output_stream: Option<StreamThread>,
    output_config: cpal::SupportedStreamConfig,
    block_size: Arc<AtomicUsize>,
    playing: bool,
}

impl Default for CpalStream {
    fn default() -> Self {
        Self::new(AudioBackend::Default, AudioDevice::Default)
    }
}

impl CpalStream {
    pub fn new(backend: AudioBackend, output_device: AudioDevice) -> Self {
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
            AudioDevice::Index(index) => host.output_devices().unwrap().nth(index).unwrap(),
            AudioDevice::Name(name) => host
                .output_devices()
                .unwrap()
                .find(|d| d.name().unwrap().contains(&name))
                .unwrap(),
        };

        let output_config = output_device.default_output_config().unwrap();

        let block_size = Arc::new(AtomicUsize::new(512)); // initialize with a default block size

        Self {
            output_device: Arc::new(output_device),
            output_stream: None,
            output_config: output_config.into(),
            block_size: block_size.clone(),
            playing: false,
        }
    }
}

fn build_output_stream<T: cpal::SizedSample + cpal::FromSample<f32> + Send + 'static>(
    graph: Graph,
    output_device: &cpal::Device,
    config: &cpal::StreamConfig,
    block_size: Arc<AtomicUsize>,
) -> cpal::Stream {
    let channels = config.channels as usize;
    output_device
        .build_output_stream(
            config,
            move |data: &mut [T], _| {
                let new_block_size = data.len() / channels;
                let old_block_size = block_size.load(Ordering::Relaxed);
                if new_block_size != old_block_size {
                    if new_block_size > old_block_size {
                        graph.allocate(graph.sample_rate(), new_block_size);
                    } else {
                        graph.resize_buffers(graph.sample_rate(), new_block_size);
                    }
                    block_size.store(new_block_size, Ordering::Relaxed);
                }

                graph.with_inner(|graph| {
                    graph.process().unwrap();
                    for output_channel in 0..channels {
                        let buffer = graph.get_output(output_channel);
                        let Some(SignalBuffer::Float(buffer)) = buffer else {
                            continue;
                        };
                        for (j, &sample) in buffer[..new_block_size].iter().enumerate() {
                            let sample = f32::from_repr(sample.unwrap_or_default());
                            data[j * channels + output_channel] = sample.to_sample();
                        }
                    }
                });
            },
            |err| {
                eprintln!("Output stream error: {}", err);
            },
            None,
        )
        .expect("Failed to build output stream")
}

impl AudioStream for CpalStream {
    fn sample_rate(&self) -> f32 {
        self.output_config.sample_rate().0 as f32
    }

    fn block_size(&self) -> usize {
        self.block_size.load(Ordering::Relaxed)
    }

    fn input_channels(&self) -> usize {
        0
    }

    fn output_channels(&self) -> usize {
        self.output_config.channels() as usize
    }

    fn spawn(&mut self, graph: &Graph) -> GraphRunResult<()> {
        let sample_format = self.output_config.sample_format();
        let output_config = self.output_config.config();
        let output_device = self.output_device.clone();
        let block_size = self.block_size.clone();
        graph.allocate(self.sample_rate(), self.block_size());
        let graph = graph.clone();
        let output_stream = move || match sample_format {
            cpal::SampleFormat::F32 => {
                build_output_stream::<f32>(graph, &output_device, &output_config, block_size)
            }
            cpal::SampleFormat::F64 => {
                build_output_stream::<f64>(graph, &output_device, &output_config, block_size)
            }
            cpal::SampleFormat::I8 => {
                build_output_stream::<i8>(graph, &output_device, &output_config, block_size)
            }
            cpal::SampleFormat::I16 => {
                build_output_stream::<i16>(graph, &output_device, &output_config, block_size)
            }
            cpal::SampleFormat::I32 => {
                build_output_stream::<i32>(graph, &output_device, &output_config, block_size)
            }
            cpal::SampleFormat::I64 => {
                build_output_stream::<i64>(graph, &output_device, &output_config, block_size)
            }
            cpal::SampleFormat::U8 => {
                build_output_stream::<u8>(graph, &output_device, &output_config, block_size)
            }
            cpal::SampleFormat::U16 => {
                build_output_stream::<u16>(graph, &output_device, &output_config, block_size)
            }
            cpal::SampleFormat::U32 => {
                build_output_stream::<u32>(graph, &output_device, &output_config, block_size)
            }
            cpal::SampleFormat::U64 => {
                build_output_stream::<u64>(graph, &output_device, &output_config, block_size)
            }

            _ => panic!("Unsupported sample format for output stream"),
        };

        self.output_stream = Some(StreamThread::spawn(output_stream));

        Ok(())
    }

    fn play(&mut self) -> GraphRunResult<()> {
        if !self.playing {
            if let Some(ref stream) = self.output_stream {
                stream.play()?;
            } else {
                return Err(GraphRunError::StreamNotSpawned);
            }
            self.playing = true;
        }
        Ok(())
    }

    fn pause(&mut self) -> GraphRunResult<()> {
        if self.playing {
            if let Some(ref stream) = self.output_stream {
                stream.pause()?;
            } else {
                return Err(GraphRunError::StreamNotSpawned);
            }
            self.playing = false;
        }
        Ok(())
    }

    fn stop(&mut self) -> GraphRunResult<()> {
        if self.playing {
            if let Some(ref stream) = self.output_stream {
                stream.stop()?;
            } else {
                return Err(GraphRunError::StreamNotSpawned);
            }
            self.playing = false;
        }
        Ok(())
    }
}
