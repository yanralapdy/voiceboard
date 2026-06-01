use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleFormat, Stream, StreamConfig};
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const SILENCE_THRESHOLD: f32 = 0.015;

pub fn list_devices() -> Result<Vec<(usize, String)>> {
    let host = cpal::default_host();
    let devices = host.input_devices()?;
    Ok(devices
        .filter_map(|d| d.name().ok())
        .enumerate()
        .collect())
}

fn resolve_device(device_id: Option<usize>) -> Result<Device> {
    let host = cpal::default_host();
    if let Some(id) = device_id {
        host.input_devices()?
            .enumerate()
            .find(|(i, _)| *i == id)
            .map(|(_, d)| d)
            .context("specified input device not found")
    } else {
        host.default_input_device()
            .context("no default input device found")
    }
}

pub fn record(device_id: Option<usize>, silence_timeout: Option<f32>) -> Result<Vec<f32>> {
    let device = resolve_device(device_id)?;
    let config = device
        .default_input_config()
        .context("failed to get default input config")?;
    let sample_rate = config.sample_rate().0;
    let channels = config.channels() as usize;
    let stream_cfg: StreamConfig = config.clone().into();

    let recorded: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
    let recorded_clone = recorded.clone();
    let stop = Arc::new(AtomicBool::new(false));
    let err_fn = |err| eprintln!("audio error: {err}");

    let stream = build_stream(&device, &stream_cfg, config.sample_format(), recorded_clone, err_fn)?;
    stream.play()?;

    if let Some(timeout) = silence_timeout {
        print!("  Recording… auto-stop after {}s of silence. Press Enter to stop early.", timeout);
        io::stdout().flush()?;

        let stop_silence = stop.clone();
        let recorded_silence = recorded.clone();
        let max_silence = (timeout * sample_rate as f32) as usize;

        thread::spawn(move || {
            let mut silence_start = 0usize;
            loop {
                thread::sleep(Duration::from_millis(100));
                if stop_silence.load(Ordering::Relaxed) {
                    break;
                }
                let guard = match recorded_silence.lock() {
                    Ok(g) => g,
                    Err(_) => continue,
                };
                let len = guard.len();
                if len < 1600 {
                    drop(guard);
                    continue;
                }
                let start = len.saturating_sub(3200);
                let rms = (guard[start..len].iter().map(|&s| s * s).sum::<f32>() / (len - start) as f32).sqrt();
                drop(guard);

                if rms < SILENCE_THRESHOLD {
                    if len - silence_start >= max_silence {
                        stop_silence.store(true, Ordering::Relaxed);
                        break;
                    }
                } else {
                    silence_start = len;
                }
            }
        });

        let stop_stdin = stop.clone();
        thread::spawn(move || {
            let mut buf = String::new();
            io::stdin().read_line(&mut buf).ok();
            stop_stdin.store(true, Ordering::Relaxed);
        });

        while !stop.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(50));
        }
    } else {
        print!("▶ Recording… press Enter to stop");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        stop.store(true, Ordering::Relaxed);
    }

    drop(stream);
    process_audio(recorded, sample_rate, channels)
}

fn build_stream(
    device: &Device,
    config: &StreamConfig,
    format: SampleFormat,
    recorded: Arc<Mutex<Vec<f32>>>,
    err_fn: impl FnMut(cpal::StreamError) + Send + 'static,
) -> Result<Stream> {
    let stream = match format {
        SampleFormat::F32 => device.build_input_stream(
            config,
            move |data: &[f32], _: &_| {
                recorded.lock().unwrap().extend_from_slice(data);
            },
            err_fn,
            None,
        )?,
        SampleFormat::I16 => device.build_input_stream(
            config,
            move |data: &[i16], _: &_| {
                let mut g = recorded.lock().unwrap();
                for &s in data {
                    g.push(s as f32 / i16::MAX as f32);
                }
            },
            err_fn,
            None,
        )?,
        SampleFormat::U16 => device.build_input_stream(
            config,
            move |data: &[u16], _: &_| {
                let mut g = recorded.lock().unwrap();
                for &s in data {
                    g.push((s as f32 - 32768.0) / 32768.0);
                }
            },
            err_fn,
            None,
        )?,
        _ => anyhow::bail!("unsupported sample format {format}"),
    };
    Ok(stream)
}

fn process_audio(
    recorded: Arc<Mutex<Vec<f32>>>,
    sample_rate: u32,
    channels: usize,
) -> Result<Vec<f32>> {
    let raw = recorded.lock().unwrap().clone();
    if raw.is_empty() {
        anyhow::bail!("no audio captured");
    }
    let mono = if channels > 1 {
        raw.chunks(channels)
            .map(|ch| ch.iter().sum::<f32>() / channels as f32)
            .collect()
    } else {
        raw
    };
    Ok(resample(&mono, sample_rate, 16000))
}

fn resample(input: &[f32], from: u32, to: u32) -> Vec<f32> {
    if from == to {
        return input.to_vec();
    }
    let ratio = from as f64 / to as f64;
    let out_len = (input.len() as f64 / ratio) as usize;
    let mut out = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let src = i as f64 * ratio;
        let lo = src.floor() as usize;
        let frac = src.fract() as f32;
        if lo + 1 < input.len() {
            out.push(input[lo] + (input[lo + 1] - input[lo]) * frac);
        } else if lo < input.len() {
            out.push(input[lo]);
        }
    }
    out
}
