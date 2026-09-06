use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc};
use std::thread;
use crossbeam_channel::Receiver;
use ringbuf::{HeapRb};
use crate::audio_decoder::AudioChunk;

pub struct AudioPlayer {
    _stream: cpal::Stream,
    samples_played: Arc<AtomicU64>,
    sample_rate: u32,
}

impl AudioPlayer {
    pub fn new(
        chunk_rx: Receiver<AudioChunk>,
        target_sample_rate: u32,
        channels: u16,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // 1. Get default audio output host & device
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or("No default audio output device found")?;

        let config = cpal::StreamConfig {
            channels,
            sample_rate: cpal::SampleRate(target_sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        // Create a lock free ring buffer (~0.5s worth of samples)
        let buffer_capacity = (target_sample_rate * channels as u32) as usize / 2;
        let ring_buffer = HeapRb::<f32>::new(buffer_capacity);
        let (mut producer, mut consumer) = ring_buffer.split();

        // Atomic counter tracking total samples played for the Master Clock
        let samples_played = Arc::new(AtomicU64::new(0));
        let sample_played_clone = Arc::clone(&samples_played);

        // Background thread: Drain chunks from 'AudioDecoder' into 'audio_buffer'
        thread::spawn(move || {
            while let Ok(chunk) = chunk_rx.recv() {
                let mut read_idx = 0;
                while read_idx < chunk.samples.len() {
                    // Push as many samples as fit without blocking
                    let written = producer.push_slice(&chunk.samples[read_idx..]);
                    read_idx += written;
                    if read_idx < chunk.samples.len() {
                        // Buffer is full: yield thread briefly
                        thread::sleep(std::time::Duration::from_millis(5));
                    }
                }
            }
        });

        // Hardware Audio stream Callback
        let stream = device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let read = consumer.pop_slice(data);

                if read < data.len() {
                    data[read..].fill(0.0);
                }

                // Update played frame count
                let frames_rendered = (read / channels as usize) as u64;
                sample_played_clone.fetch_add(frames_rendered, Ordering::Relaxed);
            },
            move |err| {
                eprintln!("Audio playback error: {:?}", err);
            },
            None
        )?;

        // Start playback
        stream.play()?;

        Ok(Self {
            _stream: stream,
            samples_played,
            sample_rate: target_sample_rate
        })
    }

    // Returns the current audio time in seconds (The Master Clock)
    pub fn get_time_seconds(&self) -> f64 {
        let played = self.samples_played.load(Ordering::Relaxed);
        played as f64 / self.sample_rate as f64
    }
}