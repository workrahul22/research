use ffmpeg_next as ffmpeg;
use crossbeam_channel::Receiver;
use std::thread;

pub struct DecodedFrame {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub pts_seconds: f64,
}

pub struct VideoDecoder {
    pub frame_rx: Receiver<DecodedFrame>,
}

impl VideoDecoder {
    pub fn new(
        packet_rx: Receiver<ffmpeg::Packet>,
        decoder_ctx: ffmpeg::decoder::Video,
        time_base: ffmpeg::Rational,
        target_width: u32,
        target_height: u32,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let (frame_tx, frame_rx) = crossbeam_channel::bounded::<DecodedFrame>(30);
        let mut decoder = decoder_ctx;

        let time_base_factor = time_base.numerator() as f64 / time_base.denominator() as f64;

        thread::spawn(move || {
            let mut decoded_yuv_frame = ffmpeg::frame::Video::empty();
            let mut rgb_frame = ffmpeg::frame::Video::empty();

            // Setup YUV420P -> RGB24 Software Scaler
            let mut scaler = match ffmpeg::software::scaling::Context::get(
                decoder.format(),
                decoder.width(),
                decoder.height(),
                ffmpeg::format::Pixel::RGB24,
                target_width,
                target_height,
                ffmpeg::software::scaling::Flags::BILINEAR,
            ){
                Ok(s) => s,
                Err(_) => return,
            };

            while let Ok(packet) = packet_rx.recv() {
                // Send compressed packet to decoded
                if decoder.send_packet(&packet).is_ok() {
                    // Pull decoded YUV frames out of decoder
                    while decoder.receive_frame(&mut decoded_yuv_frame).is_ok() {
                        // Convert PTS ticks to floating seconds
                        let pts_seconds = decoded_yuv_frame
                            .pts()
                            .map(|ptx| ptx as f64 * time_base_factor)
                            .unwrap_or(0.0);

                        // Scale YUV frame to RGB24
                        if scaler.run(&decoded_yuv_frame, &mut rgb_frame).is_ok() {
                            let frame_data = rgb_frame.data(0).to_vec();

                            let frame = DecodedFrame {
                                data: frame_data,
                                width: target_width,
                                height: target_height,
                                pts_seconds
                            };

                            if frame_tx.send(frame).is_err() {
                                return; // Receiver dropped, terminate thread
                            }
                        }
                    }
                }
            }
        });

        Ok(Self {frame_rx})
    }
}