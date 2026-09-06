use crossbeam_channel::{bounded, Receiver, Sender};
use ffmpeg_next as ffmpeg;
use std::path::PathBuf;
use std::thread;

// what is PTS?
// In video and audio streaming, PTS is a integer timestamp attached to every packet 
// and frame that tells the player exactly when that frame should be presented to the 
// user on screen or played through the speakers.
// Convert PTS to seconds
// Time in Seconds = packet.pts() * stream.time_base();
pub struct Demuxer {
    pub video_rx: Receiver<ffmpeg::Packet>,
    pub audio_rx: Receiver<ffmpeg::Packet>,
    pub video_stream_index: Option<usize>,
    pub audio_stream_index: Option<usize>,
}

impl Demuxer {
    pub fn new(path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        // Initialize ffmpeg library
        ffmpeg::init()?;

        // Open input file container
        let mut ictx = ffmpeg::format::input(&path)?;

        let video_stream_index = ictx
            .streams()
            .best(ffmpeg::media::Type::Video)
            .map(|s| s.index());

        let audio_stream_index = ictx
            .streams()
            .best(ffmpeg::media::Type::Audio)
            .map(|s| s.index());

        // Bounded queue prevent unconstrained RAM usage during demuxing
        let (video_tx, video_rx): (Sender<ffmpeg::Packet>, Receiver<ffmpeg::Packet>) = bounded(128);
        let (audio_tx, audio_rx): (Sender<ffmpeg::Packet>, Receiver<ffmpeg::Packet>) = bounded(256);

        let v_idx = video_stream_index;
        let a_idx = audio_stream_index;

        // spawn background thread for demuxing packets from file
        thread::spawn(move || {
            for (stream, packet) in ictx.packets() {
                let stream_index = stream.index();

                if Some(stream_index) == v_idx {
                    if video_tx.send(packet).is_err() {
                        break; // Receiver disconnedted, stop demuxing
                    }
                } else if Some(stream_index) == a_idx {
                    if audio_tx.send(packet).is_err() {
                        break;
                    }
                }
            }
        });
        Ok(Self {
            video_rx,
            audio_rx,
            video_stream_index,
            audio_stream_index
        })
    }

    pub fn video_decoder(&self, path: &std::path::Path) -> Result<ffmpeg::decoder::Video, Box<dyn std::error::Error>> {
        let ictx = ffmpeg::format::input(&path)?;
        if let Some(index) = self.video_stream_index {
            let stream = ictx.stream(index).ok_or("Video stream not found")?;
            let context = ffmpeg::codec::context::Context::from_parameters(stream.parameters())?;
            let decoder = context.decoder().video()?;
            Ok(decoder)
        } else {
            Err("No video stream found".into())
        }
    }

    pub fn video_time_base(&self, path: &std::path::Path) -> Result<ffmpeg::Rational, Box<dyn std::error::Error>> {
        let ictx = ffmpeg::format::input(&path)?;
        if let Some(index) = self.video_stream_index {
            let stream = ictx.stream(index).ok_or("Video stream not found")?;
            Ok(stream.time_base())
        } else {
            Err("No video stream found".into())
        }
    }

    pub fn audio_decoder(
        &self,
        path: &std::path::Path,
    ) -> Result<ffmpeg::decoder::Audio, Box<dyn std::error::Error>> {
        let ictx = ffmpeg::format::input(&path)?;
        if let Some(index) = self.audio_stream_index {
            let stream = ictx.stream(index).ok_or("Audio Stream not found")?;
            let context = ffmpeg::codec::context::Context::from_parameters(stream.parameters())?;
            let decoder = context.decoder().audio()?;
            Ok(decoder)
        } else {
            Err("No audi stream found in container".into())
        }
    }

    pub fn audio_time_base(
        &self,
        path: &std::path::Path,
    ) -> Result<ffmpeg::Rational, Box<dyn std::error::Error>> {
        let ictx = ffmpeg::format::input(&path)?;
        if let Some(index) = self.audio_stream_index {
            let stream = ictx.stream(index).ok_or("Audio stream not found")?;
            Ok(stream.time_base())
        } else {
            Err("No audio stream found in container".into())
        }
    }
}