use ffmpeg_next as ffmpeg;
use std::thread;
use crossbeam_channel::Receiver;

pub struct AudioChunk {
    pub samples: Vec<f32>, // Interleaved L/R f32 PCM samples
    pub pts_seconds: f64,
}

pub struct AudioDecoder {
    pub audio_rx: Receiver<AudioChunk>,
}

impl AudioDecoder {
    pub fn new(
        packet_rx: Receiver<ffmpeg::Packet>,
        decoder_ctx: ffmpeg::decoder::Audio,
        time_base: ffmpeg::Rational,
        target_sample_rate: u32,
        target_channles: u16,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let (audio_tx, audio_rx) = crossbeam_channel::bounded::<AudioChunk>(60);

        let mut decoder = decoder_ctx;
        let time_base_factor = time_base.numerator() as f64 / time_base.denominator() as f64;

        // Capture input metadata needed to configure resampler inside the thread
        let src_format = decoder.format();
        let src_rate = decoder.rate();

        thread::spawn(move || {
            
            // Capture input metadata needed to configure resampler inside the thread
            let src_channel_layout = decoder.channel_layout();

            // Standard output layout for cpal: Stereo or Mono packed f32
            let target_channel_layout = if target_channles == 1 {
                ffmpeg::util::channel_layout::ChannelLayout::MONO
            } else {
                ffmpeg::util::channel_layout::ChannelLayout::STEREO
            };

            // Instantiate the Audio Resampler INSIDE the worker thread to avoid raw pointer thread bound
            let mut resampler = match ffmpeg::software::resampling::Context::get(
                src_format,
                src_channel_layout,
                src_rate,
                ffmpeg::format::Sample::F32(ffmpeg::format::sample::Type::Packed),
                target_channel_layout,
                target_sample_rate
            ) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Failed to initialize audio resampler: {:?}", e);
                    return;
                }
            };

            let mut decoded_audio_frame = ffmpeg::frame::Audio::empty();
            let mut resampled_audio_frame = ffmpeg::frame::Audio::empty();

            while let Ok(packet) = packet_rx.recv() {
                if decoder.send_packet(&packet).is_ok() {
                    while decoder.receive_frame(&mut decoded_audio_frame).is_ok() {
                        let pts_seconds = decoded_audio_frame
                            .pts()
                            .map(|pts| pts as f64 * time_base_factor)
                            .unwrap_or(0.0);

                        // Resample audio to 32 bit float packed stream
                        if resampler
                            .run(&decoded_audio_frame, &mut resampled_audio_frame)
                            .is_ok()
                        {
                            // Extract byte buffer and convert to f32 slice
                            let data_bytes = resampled_audio_frame.data(0);
                            let samples_f32: &[f32] = unsafe {
                                std::slice::from_raw_parts(
                                    data_bytes.as_ptr() as *const f32,
                                    data_bytes.len() / std::mem::size_of::<f32>(),
                                )
                            };

                            let chunk = AudioChunk {
                                samples: samples_f32.to_vec(),
                                pts_seconds,
                            };

                            if audio_tx.send(chunk).is_err() {
                                return; // Downstream dropped
                            }
                        }
                    }
                }
            }
        });

        Ok(Self { audio_rx })
    }
}