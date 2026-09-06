FFMPEG

Libraries:
1. libavformat: Reads and writes container formats (AVI, MKV, MP4, ...)
2. libavcodec: Read and write codecs (H.264, H.265, VP9, ...)
3. libavfilter: Various filters for video and audio.

Container Format: It contains media data

1. MP4: MPEG-4 Part 14 Container for H.264, h.265, AAC audio, ...
2. MKV: Versatile container for any media format.
3. WebM: Subset of MKV, usage in web streaming.
4. AVI: Legacy Container.

To check the list of formats. you can run the command:
$ ffmpeg -formats

Codecs: Coder / Decoder
1. Specification on how to code and decode a video, audio, ...

To see the list of codecs you can run this command:
$ ffmpeg -formats

Most Important Lossy Codecs
1. H.262 / MPEG-2 Part H: Broadcasting, TV, used for backward compatibility
2. H.264 / MPEG-4 Part 10: The de-facto standard for video encoding today.
3. H.265 / HEVC / MPEG-H: Successor of H.264, up to 50% better quality.
4. MP3 / MPEG-2 Audio Layer III: Used to be the de-facto audio coading standard
5. AAC / ISO / IEC 14496-3:2009: Advanced audio coading standard.

Competitors that are royality free
1. VP8: Free, Open source codec from Google(not so much in use anymore)
2. VP9: Successor of VP8, almost as good as H.265
3. AV1: A successor to VP9, claims to be better than H.265

Most Importand Lossless Codec: Lossless codecs are useful for archival, editing, ...
1. Raw YUV, HuffYUV, FFV1, ffvhuff ...
2. Raw PCM, FLAC, ALAC, ...

Also Visually lossless codec exists
1. Apple ProRes, Avid DNxHD, JPEG2000, high quality H.264/H.265,...

Encoders: 

Encoders are the actual software that outputs a codec-compliment bitstream.
Encoder can vary in quality and performance, some are better than others (some are free and some are not)

1. libx264 : most popular free and open source H.264 encoder.
2. NVENC: NVIDIA GPU based H.264 encoder
3. libx265: free and open source HEVC encoder.
4. libvpx: vp8 and vp9 encoder from Google.
5. libaom: AV1 encoder.
6. libfdk-aac: AAC encoder
7. aac: native FFmpeg AAC encoder

To check the encoders supported in FFMPEG use this command
$ ffmpeg -encoders

Pixel Formats:
Representation of raw pixel in video stream
Specifies order of lima/color components and chroma subsampling

To check supported pixel formats
$ ffmpeg -pix_fmts


