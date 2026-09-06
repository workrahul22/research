// Declare the module (tell rust to look for demuxer.rs)
mod audio_decoder;
mod audio_player;
mod demuxer;
mod video_decoder;

// Bring the Demuxer struct into the scope
use audio_decoder::AudioDecoder;
use audio_player::AudioPlayer;
use demuxer::Demuxer;
use video_decoder::VideoDecoder;
use minifb::{Key, Window, WindowOptions};
use std::path::PathBuf;

fn rgb24_to_u32_buffer(rgb_bytes: &[u8], pixel_count: usize) -> Vec<u32> {
    let mut buffer = vec![0u32; pixel_count];
    for (i, chunk) in rgb_bytes.chunks_exact(3).enumerate() {
        if i < buffer.len() {
            let r = chunk[0] as u32;
            let g = chunk[1] as u32;
            let b = chunk[2] as u32;
            buffer[i] = (r << 16) | (g << 8) | b;
        }
    }
    buffer
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from("test_video.mp4");
    let sample_rate = 44100;
    let channels = 2;

    //1. Intitialize Demuxer
    let demuxer = Demuxer::new(path.clone())?;

    // 2. Video Decoder
    let v_ctx = demuxer.video_decoder(&path)?;
    let v_tb = demuxer.video_time_base(&path)?;
    let width = v_ctx.width() * 3;
    let height = v_ctx.height() * 3;
    let video_decoder = VideoDecoder::new(
        demuxer.video_rx.clone(),
        v_ctx,
        v_tb,
        width,
        height,
    )?;

    // 3. Audio Decoder & Player
    let _audio_player = if demuxer.audio_stream_index.is_some() {
        let a_ctx = demuxer.audio_decoder(&path)?;
        let a_tb = demuxer.audio_time_base(&path)?;
        let audio_decoder = AudioDecoder::new(demuxer.audio_rx, a_ctx, a_tb, sample_rate, channels)?;
        let player = AudioPlayer::new(audio_decoder.audio_rx, sample_rate, channels)?;
        println!("Audio output initialized successfully!");
        Some(player)
    } else {
        None
    };

    // Open a native desktop window using minifb
    let mut window = Window::new(
        "Rust Video Player",
        width as usize,
        height as usize,
        WindowOptions::default(),
    )?;

    // Limit Window refresh rate to ~60 FPS
    window.set_target_fps(60);

    println!("Rendering video to window. Press ESC to quit.");

    // Render loop
    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Receive next decoded frame from the decoded channel
        if let Ok(frame) = video_decoder.frame_rx.try_recv() {
            let u32_buffer = rgb24_to_u32_buffer(&frame.data, (width*height) as usize);
            // copy buffer to window surface
            window.update_with_buffer(&u32_buffer, width as usize, height as usize)?;
        } else {
            // If no frame is ready yet. Keep window event loop responsive
            window.update();
        }
    } 
    

    Ok(())
}