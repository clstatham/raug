use std::f32::consts::PI;

use raug::prelude::*;

#[derive(Clone, Default)]
pub struct SampleStorage {
    buf: Vec<f32>,
    sample_rate: f32,
}

impl SampleStorage {
    pub fn load(path: &str) -> Result<Self, hound::Error> {
        let reader = hound::WavReader::open(path)?;
        let spec = reader.spec();

        let channels = spec.channels as usize;
        let sample_rate = spec.sample_rate as f32;
        let buf = match spec.sample_format {
            hound::SampleFormat::Int => {
                let buf = reader
                    .into_samples::<i32>()
                    .step_by(channels) // take only one channel
                    .collect::<Result<Vec<_>, _>>()?;
                buf.into_iter()
                    .map(|s| s as f32 / (1 << spec.bits_per_sample) as f32)
                    .collect()
            }
            hound::SampleFormat::Float => {
                reader
                    .into_samples::<f32>()
                    .step_by(channels) // take only one channel
                    .collect::<Result<Vec<_>, _>>()?
            }
        };

        Ok(Self { buf, sample_rate })
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    pub fn resample(&mut self, new_sample_rate: f32) {
        if new_sample_rate != self.sample_rate && new_sample_rate > 0.0 {
            self.buf = resample(&self.buf, self.sample_rate, new_sample_rate);
            self.sample_rate = new_sample_rate;
        }
    }

    #[inline]
    pub fn get_interpolated(&self, index: f32) -> f32 {
        let floor = index.floor() as usize;
        let ceil = index.ceil() as usize;
        let t = index - floor as f32;
        let floor_value = if floor < self.buf.len() {
            self.buf[floor]
        } else {
            0.0
        };
        let ceil_value = if ceil < self.buf.len() {
            self.buf[ceil]
        } else {
            0.0
        };
        floor_value * (1.0 - t) + ceil_value * t
    }
}

#[processor(allocate = sample_allocate)]
#[allow(unused)]
pub fn sample(
    env: ProcEnv,
    #[state] storage: &mut SampleStorage,
    #[input] index: &f32,
    #[input] wrap: &bool,
    #[output] out: &mut f32,
    #[output] length: &mut f32,
) -> ProcResult<()> {
    let index = if *wrap {
        *index % storage.buf.len() as f32
    } else {
        *index
    };
    *out = storage.get_interpolated(index);

    *length = storage.len() as f32;

    Ok(())
}

fn sinc(x: f32) -> f32 {
    if x == 0.0 {
        1.0
    } else {
        (PI * x).sin() / (PI * x)
    }
}

fn hann(x: f32, half_width: f32) -> f32 {
    0.5 * (1.0 + (PI * x / half_width).cos())
}

fn windowed_sinc(x: f32, half_width: f32) -> f32 {
    if x.abs() < half_width {
        sinc(x) * hann(x, half_width)
    } else {
        0.0
    }
}

/// Resamples the input buffer using a high-quality sinc interpolation.
fn resample(input: &[f32], input_rate: f32, output_rate: f32) -> Vec<f32> {
    if input_rate == 0.0 || output_rate == 0.0 {
        return input.to_vec();
    }

    let ratio = input_rate / output_rate;
    let filter_width = 32.0;
    let half_width = filter_width / 2.0;
    let input_len = input.len() as isize;
    let output_len = (input_len as f32 * output_rate / input_rate).ceil() as usize;
    let mut output = vec![0.0; output_len];

    for (i, output_sample) in output.iter_mut().enumerate() {
        let t = i as f32 * ratio;
        let t_int = t.floor() as isize;

        let mut sample = 0.0;

        for n in (t_int - half_width as isize)..=(t_int + half_width as isize) {
            if n >= 0 && n < input_len {
                let x = t - n as f32;
                let sinc_value = windowed_sinc(x, half_width);
                sample += sinc_value * input[n as usize];
            }
        }

        *output_sample = sample;
    }

    output
}

fn sample_allocate(proc: &mut Sample, sample_rate: f32, _block_size: usize) {
    proc.storage.resample(sample_rate);
}

impl Sample {
    pub fn load(path: &str) -> Result<Self, hound::Error> {
        let storage = SampleStorage::load(path)?;
        Ok(Self {
            storage,
            ..Default::default()
        })
    }

    pub fn length(&self) -> f32 {
        self.storage.len() as f32
    }
}

impl Default for Sample {
    fn default() -> Self {
        Self {
            storage: Default::default(),
            index: 0.0,
            wrap: false,
        }
    }
}

#[derive(Clone, Default, PartialEq)]
enum OneShotState {
    #[default]
    Waiting,
    Playing,
}

#[processor(allocate = one_shot_allocate)]
#[allow(unused)]
pub fn one_shot(
    #[state] storage: &mut SampleStorage,
    #[state] play_head: &mut f32,
    #[state] state: &mut OneShotState,
    #[input] trig: &bool,
    #[input] rate: &f32,
    #[output] out: &mut f32,
    #[output] length: &mut f32,
) -> ProcResult<()> {
    *length = storage.len() as f32;

    if *trig {
        *play_head = 0.0;
        *state = OneShotState::Playing;
    }

    if *state == OneShotState::Waiting {
        return Ok(());
    }

    if *play_head >= storage.len() as f32 {
        *out = 0.0;
        *state = OneShotState::Waiting;
    } else {
        *out = storage.get_interpolated(*play_head);
    }

    *play_head += *rate;

    Ok(())
}

impl OneShot {
    pub fn load(path: &str) -> Result<Self, hound::Error> {
        let storage = SampleStorage::load(path)?;
        Ok(Self {
            storage,
            ..Default::default()
        })
    }
}

impl Default for OneShot {
    fn default() -> Self {
        Self {
            storage: Default::default(),
            play_head: 0.0,
            state: OneShotState::Waiting,
            trig: false,
            rate: 1.0,
        }
    }
}

fn one_shot_allocate(proc: &mut OneShot, sample_rate: f32, _block_size: usize) {
    proc.storage.resample(sample_rate);
}
