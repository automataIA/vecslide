//! Muxing of Opus packets into an OGG container (RFC 7845).
//!
//! This module is **pure-Rust** (no web-sys dependency): it can be
//! unit-tested on native targets and reused both by the online synthesis
//! path (see [`crate::tts::synth`]) and by the `.vecslide` export that
//! concatenates multiple streams into a single master (see [`crate::export`]).
//!
//! Layout of a valid Opus stream:
//! 1. `OpusHead` page (ID header, RFC 7845 §5.1) — implicit BOS flag.
//! 2. `OpusTags` page (comment header, RFC 7845 §5.2).
//! 3. Subsequent pages with Opus audio packets; the `granule_position`
//!    is the cumulative number of samples **at the reference sample rate
//!    of 48 kHz**, regardless of the actual input sample rate.

use std::io::Cursor;

use ogg::reading::PacketReader;
use ogg::writing::{PacketWriter, PacketWriteEndInfo};

/// An Opus packet ready for muxing.
///
/// `samples_48k` is the packet duration expressed as the number of samples
/// at 48 kHz (the Opus reference rate). For standard 20 ms frames this is
/// `960`. For 10 ms frames it is `480`, for 40 ms frames it is `1920`, etc.
#[derive(Debug, Clone)]
pub struct OpusPacket {
    pub data: Vec<u8>,
    pub samples_48k: u64,
}

/// Arbitrary but stable serial number for our single logical stream.
/// With a single Opus stream in a `.ogg` file there are no conflicts.
const STREAM_SERIAL: u32 = 0x7665_6373; // "vecs" in ASCII

/// Standard pre-skip for Opus (80 ms at 48 kHz). The exact value would depend
/// on the encoder configuration, but 3840 is the recommended default from
/// RFC 7845 §4.1 for SILK warm-up and is always safe.
const PRE_SKIP: u16 = 3840;

/// Vendor string for `OpusTags` (RFC 7845 §5.2).
const VENDOR: &str = "vecslide-app";

/// Builds the `OpusHead` header (19 bytes, RFC 7845 §5.1).
fn build_opus_head(channels: u8, input_sample_rate: u32) -> Vec<u8> {
    let mut head = Vec::with_capacity(19);
    head.extend_from_slice(b"OpusHead");
    head.push(1); // version
    head.push(channels);
    head.extend_from_slice(&PRE_SKIP.to_le_bytes());
    head.extend_from_slice(&input_sample_rate.to_le_bytes());
    head.extend_from_slice(&0i16.to_le_bytes()); // output gain (Q7.8 dB, 0 = neutral)
    head.push(0); // channel mapping family 0 (mono / stereo standard)
    debug_assert_eq!(head.len(), 19);
    head
}

/// Builds the `OpusTags` comment header (RFC 7845 §5.2) without user comments.
fn build_opus_tags() -> Vec<u8> {
    let vendor_bytes = VENDOR.as_bytes();
    let mut tags = Vec::with_capacity(16 + vendor_bytes.len());
    tags.extend_from_slice(b"OpusTags");
    tags.extend_from_slice(&(u32::try_from(vendor_bytes.len()).unwrap_or(0)).to_le_bytes());
    tags.extend_from_slice(vendor_bytes);
    tags.extend_from_slice(&0u32.to_le_bytes()); // 0 user comments
    tags
}

/// Serializes `packets` as a mono/stereo OGG Opus file.
///
/// Uses `input_sample_rate` in the `OpusHead` to annotate the original
/// signal sample rate (typically 24000 Hz for Kokoro). The Opus decoder
/// will still produce 48 kHz internally: this field is informational.
pub fn write_ogg_opus(
    packets: &[OpusPacket],
    channels: u8,
    input_sample_rate: u32,
) -> Result<Vec<u8>, String> {
    let mut pw = PacketWriter::new(Cursor::new(Vec::<u8>::new()));

    // BOS page with the ID header.
    pw.write_packet(
        build_opus_head(channels, input_sample_rate),
        STREAM_SERIAL,
        PacketWriteEndInfo::EndPage,
        0,
    )
    .map_err(|e| format!("write OpusHead: {e}"))?;

    // Page with the comment header.
    pw.write_packet(
        build_opus_tags(),
        STREAM_SERIAL,
        PacketWriteEndInfo::EndPage,
        0,
    )
    .map_err(|e| format!("write OpusTags: {e}"))?;

    // Audio pages: cumulative granule at 48 kHz; flush EndStream on the last one.
    let mut cumulative: u64 = 0;
    let last = packets.len().saturating_sub(1);
    for (i, pkt) in packets.iter().enumerate() {
        cumulative = cumulative.saturating_add(pkt.samples_48k);
        let info = if i == last {
            PacketWriteEndInfo::EndStream
        } else {
            PacketWriteEndInfo::NormalPacket
        };
        pw.write_packet(pkt.data.clone(), STREAM_SERIAL, info, cumulative)
            .map_err(|e| format!("write opus packet {i}: {e}"))?;
    }

    Ok(pw.into_inner().into_inner())
}

/// Concatenates multiple OGG Opus streams into a single master, preserving
/// audio packets but rewriting headers, serial number, and granule position.
///
/// Intended use: during `.vecslide` export the master `audio/voice.ogg`
/// is the concatenation of per-slide files, with slide boundaries aligned
/// to page endings to allow precise seeking.
///
/// `channels` and `input_sample_rate` must match those used to generate
/// the individual streams — typically 1 mono and 24000 Hz.
pub fn concat_ogg_streams(
    streams: &[&[u8]],
    channels: u8,
    input_sample_rate: u32,
) -> Result<Vec<u8>, String> {
    let mut pw = PacketWriter::new(Cursor::new(Vec::<u8>::new()));

    // A shared header for the entire master.
    pw.write_packet(
        build_opus_head(channels, input_sample_rate),
        STREAM_SERIAL,
        PacketWriteEndInfo::EndPage,
        0,
    )
    .map_err(|e| format!("write master OpusHead: {e}"))?;
    pw.write_packet(
        build_opus_tags(),
        STREAM_SERIAL,
        PacketWriteEndInfo::EndPage,
        0,
    )
    .map_err(|e| format!("write master OpusTags: {e}"))?;

    let mut cumulative: u64 = 0;
    // Pre-read all audio packets from every stream to determine which
    // will be the global last one (needed to mark it `EndStream`).
    let mut all_audio: Vec<(usize, Vec<u8>, u64)> = Vec::new();
    for (si, &bytes) in streams.iter().enumerate() {
        let mut reader = PacketReader::new(Cursor::new(bytes.to_vec()));
        let mut prev_absgp: u64 = 0;
        // Discard the first 2 packets (OpusHead + OpusTags).
        for _ in 0..2 {
            match reader.read_packet().map_err(|e| format!("read header {si}: {e}"))? {
                Some(_) => {}
                None => return Err(format!("stream {si}: missing headers")),
            }
        }
        while let Some(pkt) = reader
            .read_packet()
            .map_err(|e| format!("read audio {si}: {e}"))?
        {
            let cur_absgp = pkt.absgp_page();
            // Estimate the number of samples in THIS packet as the
            // granule difference from the previous one; for "intermediate"
            // packets within a page, `absgp_page` stays constant, so we
            // use 0 (it becomes incremental at page end). This is
            // sufficient for correct cumulative granule at page level.
            let delta = cur_absgp.saturating_sub(prev_absgp);
            prev_absgp = cur_absgp;
            all_audio.push((si, pkt.data, delta));
        }
    }

    let last_idx = all_audio.len().saturating_sub(1);
    let mut prev_slide: Option<usize> = None;
    for (i, (slide_idx, data, delta_samples)) in all_audio.into_iter().enumerate() {
        cumulative = cumulative.saturating_add(delta_samples);

        // Force a page-flush when the slide changes: slide boundaries
        // remain aligned to page endings so the viewer can seek
        // precisely to each slide's `time_start`.
        let is_last = i == last_idx;
        let crossing_slide = prev_slide.is_some_and(|p| p != slide_idx);
        let info = if is_last {
            PacketWriteEndInfo::EndStream
        } else if crossing_slide {
            PacketWriteEndInfo::EndPage
        } else {
            PacketWriteEndInfo::NormalPacket
        };
        prev_slide = Some(slide_idx);

        pw.write_packet(data, STREAM_SERIAL, info, cumulative)
            .map_err(|e| format!("write master audio {i}: {e}"))?;
    }

    Ok(pw.into_inner().into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fake_packet(byte: u8, len: usize) -> OpusPacket {
        OpusPacket {
            data: vec![byte; len],
            samples_48k: 960,
        }
    }

    #[test]
    fn opus_head_is_19_bytes() {
        let head = build_opus_head(1, 24_000);
        assert_eq!(head.len(), 19);
        assert_eq!(&head[..8], b"OpusHead");
        assert_eq!(head[8], 1); // version
        assert_eq!(head[9], 1); // channels
        // pre-skip LE at offset 10
        assert_eq!(u16::from_le_bytes([head[10], head[11]]), PRE_SKIP);
        // input sample rate LE at offset 12
        assert_eq!(
            u32::from_le_bytes([head[12], head[13], head[14], head[15]]),
            24_000
        );
    }

    #[test]
    fn opus_tags_starts_with_magic() {
        let tags = build_opus_tags();
        assert_eq!(&tags[..8], b"OpusTags");
        let vendor_len =
            u32::from_le_bytes([tags[8], tags[9], tags[10], tags[11]]) as usize;
        assert_eq!(&tags[12..12 + vendor_len], VENDOR.as_bytes());
    }

    #[test]
    fn write_ogg_opus_produces_ogg_magic() {
        let packets = vec![fake_packet(0xAA, 10), fake_packet(0xBB, 10)];
        let bytes = write_ogg_opus(&packets, 1, 24_000).expect("mux ok");
        // Each OGG page starts with "OggS" (capture pattern).
        assert_eq!(&bytes[..4], b"OggS", "first page must start with OggS");
        assert!(bytes.len() > 19 + 16, "should contain header + tags + audio");
    }

    #[test]
    fn write_ogg_opus_roundtrip_through_reader() {
        let packets = vec![
            fake_packet(0x11, 8),
            fake_packet(0x22, 8),
            fake_packet(0x33, 8),
        ];
        let bytes = write_ogg_opus(&packets, 1, 24_000).expect("mux ok");

        let mut reader = PacketReader::new(Cursor::new(bytes));

        // Packet 1 = OpusHead
        let head = reader.read_packet().unwrap().unwrap();
        assert_eq!(&head.data[..8], b"OpusHead");

        // Packet 2 = OpusTags
        let tags = reader.read_packet().unwrap().unwrap();
        assert_eq!(&tags.data[..8], b"OpusTags");

        // Packets 3..5 = audio, in order
        let expected_bytes = [0x11u8, 0x22, 0x33];
        for (i, &expected) in expected_bytes.iter().enumerate() {
            let pkt = reader
                .read_packet()
                .unwrap()
                .unwrap_or_else(|| panic!("audio packet {i} missing"));
            assert!(pkt.data.iter().all(|&b| b == expected));
        }
        assert!(reader.read_packet().unwrap().is_none(), "stream should end");
    }

    #[test]
    fn empty_packet_list_still_writes_headers() {
        let bytes = write_ogg_opus(&[], 1, 24_000).expect("mux ok");
        let mut reader = PacketReader::new(Cursor::new(bytes));
        let head = reader.read_packet().unwrap().unwrap();
        assert_eq!(&head.data[..8], b"OpusHead");
        let tags = reader.read_packet().unwrap().unwrap();
        assert_eq!(&tags.data[..8], b"OpusTags");
    }

    #[test]
    fn concat_two_streams_preserves_audio_packet_count() {
        let a = write_ogg_opus(&[fake_packet(0x01, 4), fake_packet(0x02, 4)], 1, 24_000)
            .expect("a");
        let b = write_ogg_opus(&[fake_packet(0x03, 4)], 1, 24_000).expect("b");

        let master = concat_ogg_streams(&[&a, &b], 1, 24_000).expect("concat");

        let mut reader = PacketReader::new(Cursor::new(master));
        let _head = reader.read_packet().unwrap().unwrap();
        let _tags = reader.read_packet().unwrap().unwrap();

        let mut audio_first_bytes: Vec<u8> = Vec::new();
        while let Some(pkt) = reader.read_packet().unwrap() {
            audio_first_bytes.push(pkt.data[0]);
        }
        assert_eq!(audio_first_bytes, vec![0x01, 0x02, 0x03]);
    }
}
