use std::collections::HashSet;

use crate::{
    error::VecslideError,
    manifest::{Presentation, TranscriptMode},
};

/// Validates a presentation without touching the filesystem. WASM-safe.
/// Returns all errors found, not just the first.
pub fn validate(presentation: &Presentation) -> Vec<VecslideError> {
    let mut errors = Vec::new();

    // --- audio_track ---
    if matches!(&presentation.audio_track, Some(s) if s.is_empty()) {
        errors.push(VecslideError::EmptyAudioTrack);
    }

    // --- slides ---
    if presentation.slides.is_empty() {
        errors.push(VecslideError::EmptySlides);
        return errors; // cannot check further with no slides
    }

    // Unique slide ids
    let mut seen_ids: HashSet<&str> = HashSet::new();
    for slide in &presentation.slides {
        if !seen_ids.insert(slide.id.as_str()) {
            errors.push(VecslideError::DuplicateSlideId { id: slide.id.clone() });
        }
    }

    // Ascending timestamps + animations
    let n = presentation.slides.len();
    let mut prev_time = 0u64;
    for (i, slide) in presentation.slides.iter().enumerate() {
        if i > 0 && slide.time_start <= prev_time {
            errors.push(VecslideError::TimestampOrder {
                index: i,
                detail: format!(
                    "slide '{}' time_start {}ms <= previous {}ms",
                    slide.id, slide.time_start, prev_time
                ),
            });
        }

        // Upper bound: next slide's time_start (or u64::MAX for the last slide)
        let slide_end = if i + 1 < n {
            presentation.slides[i + 1].time_start
        } else {
            u64::MAX
        };

        for (j, anim) in slide.animations.iter().enumerate() {
            if anim.element_id.is_empty() {
                errors.push(VecslideError::EmptyElementId {
                    slide_id: slide.id.clone(),
                    anim_index: j,
                });
            }
            if anim.time_start < slide.time_start || anim.time_start >= slide_end {
                errors.push(VecslideError::AnimationOutOfRange { slide_id: slide.id.clone() });
            }
        }

        prev_time = slide.time_start;
    }

    // --- transcript ---
    if let Some(ref transcript) = presentation.transcript {
        // Synchronized mode requires audio_track
        if matches!(transcript.mode, TranscriptMode::Synchronized)
            && presentation.audio_track.is_none()
        {
            errors.push(VecslideError::EmptyAudioTrack);
        }

        let mut prev_end = 0u64;
        for (i, seg) in transcript.segments.iter().enumerate() {
            if seg.start_ms >= seg.end_ms {
                errors.push(VecslideError::InvalidSegment { index: i });
            }
            if i > 0 && seg.start_ms < prev_end {
                errors.push(VecslideError::OverlappingSegments { index: i });
            }
            prev_end = seg.end_ms;
        }
    }

    errors
}

/// Validates a presentation and also checks that referenced SVG files exist on disk.
/// Native only (requires filesystem access).
#[cfg(feature = "native")]
pub fn validate_with_dir(
    presentation: &Presentation,
    base_dir: &std::path::Path,
) -> Vec<VecslideError> {
    let mut errors = validate(presentation);

    for slide in &presentation.slides {
        if let Some(ref svg_file) = slide.svg_file {
            let svg_path = base_dir.join(svg_file);
            if !svg_path.exists() {
                errors.push(VecslideError::MissingSvgFile {
                    path: svg_file.clone(),
                });
            }
        }
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{Animation, Slide, Transcript, TranscriptMode, TranscriptSegment};

    fn make_slide(id: &str, time_start: u64) -> Slide {
        Slide {
            id: id.to_string(),
            svg_file: Some(format!("{id}.svg")),
            typst_file: None,
            typst_inline: None,
            time_start,
            animations: vec![],
            pointer_trail: None,
            transition: None,
        }
    }

    fn make_presentation(slides: Vec<Slide>) -> Presentation {
        Presentation {
            format_version: "1.0".to_string(),
            title: "Test".to_string(),
            author: None,
            description: None,
            date: None,
            language: None,
            audio_track: Some("audio/test.opus".to_string()),
            typst_source: None,
            slides,
            annotations: vec![],
            transcript: None,
        }
    }

    #[test]
    fn valid_presentation_has_no_errors() {
        let pres = make_presentation(vec![
            make_slide("s1", 0),
            make_slide("s2", 5000),
            make_slide("s3", 10000),
        ]);
        assert!(validate(&pres).is_empty());
    }

    #[test]
    fn empty_audio_track_is_an_error() {
        let mut pres = make_presentation(vec![make_slide("s1", 0)]);
        pres.audio_track = Some(String::new());
        let errors = validate(&pres);
        assert!(errors
            .iter()
            .any(|e| matches!(e, VecslideError::EmptyAudioTrack)));
    }

    #[test]
    fn empty_slides_is_an_error() {
        let pres = make_presentation(vec![]);
        let errors = validate(&pres);
        assert!(errors
            .iter()
            .any(|e| matches!(e, VecslideError::EmptySlides)));
    }

    #[test]
    fn non_ascending_timestamps_are_detected() {
        let pres = make_presentation(vec![
            make_slide("s1", 0),
            make_slide("s2", 10000),
            make_slide("s3", 5000), // goes backwards
        ]);
        let errors = validate(&pres);
        assert!(errors
            .iter()
            .any(|e| matches!(e, VecslideError::TimestampOrder { index: 2, .. })));
    }

    #[test]
    fn duplicate_slide_ids_are_detected() {
        let pres = make_presentation(vec![
            make_slide("s1", 0),
            make_slide("s1", 5000), // duplicate id
        ]);
        let errors = validate(&pres);
        assert!(errors
            .iter()
            .any(|e| matches!(e, VecslideError::DuplicateSlideId { .. })));
    }

    #[test]
    fn empty_element_id_in_animation_is_detected() {
        let mut slide = make_slide("s1", 0);
        slide.animations.push(Animation {
            element_id: String::new(),
            time_start: 0,
            duration: 500,
            kind: "fade-in".to_string(),
        });
        let pres = make_presentation(vec![slide]);
        let errors = validate(&pres);
        assert!(errors
            .iter()
            .any(|e| matches!(e, VecslideError::EmptyElementId { .. })));
    }

    #[test]
    fn animation_out_of_range_is_detected() {
        let mut slide = make_slide("s1", 1000);
        slide.animations.push(Animation {
            element_id: "el".to_string(),
            time_start: 500, // before slide.time_start
            duration: 100,
            kind: "fade-in".to_string(),
        });
        let pres = make_presentation(vec![slide]);
        let errors = validate(&pres);
        assert!(errors
            .iter()
            .any(|e| matches!(e, VecslideError::AnimationOutOfRange { .. })));
    }

    #[test]
    fn invalid_transcript_segment_detected() {
        let mut pres = make_presentation(vec![make_slide("s1", 0)]);
        pres.transcript = Some(Transcript {
            mode: TranscriptMode::Synchronized,
            language: "it-IT".to_string(),
            segments: vec![TranscriptSegment {
                start_ms: 1000,
                end_ms: 500, // end < start
                text: "hello".to_string(),
                slide_ref: None,
                words: vec![],
            }],
        });
        let errors = validate(&pres);
        assert!(errors
            .iter()
            .any(|e| matches!(e, VecslideError::InvalidSegment { .. })));
    }

    #[test]
    fn overlapping_transcript_segments_detected() {
        let mut pres = make_presentation(vec![make_slide("s1", 0)]);
        pres.transcript = Some(Transcript {
            mode: TranscriptMode::Synchronized,
            language: "it-IT".to_string(),
            segments: vec![
                TranscriptSegment {
                    start_ms: 0,
                    end_ms: 2000,
                    text: "first".to_string(),
                    slide_ref: None,
                    words: vec![],
                },
                TranscriptSegment {
                    start_ms: 1500, // overlaps with previous
                    end_ms: 3000,
                    text: "second".to_string(),
                    slide_ref: None,
                    words: vec![],
                },
            ],
        });
        let errors = validate(&pres);
        assert!(errors
            .iter()
            .any(|e| matches!(e, VecslideError::OverlappingSegments { .. })));
    }

    #[test]
    fn synchronized_transcript_requires_audio_track() {
        let mut pres = make_presentation(vec![make_slide("s1", 0)]);
        pres.audio_track = None;
        pres.transcript = Some(Transcript {
            mode: TranscriptMode::Synchronized,
            language: "it-IT".to_string(),
            segments: vec![],
        });
        let errors = validate(&pres);
        assert!(errors
            .iter()
            .any(|e| matches!(e, VecslideError::EmptyAudioTrack)));
    }
}
