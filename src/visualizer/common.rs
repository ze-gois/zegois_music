//! Shared canvas and note helpers for visualizers.
//!
//! Visualizers all use semitone offsets from A4. These helpers convert those
//! offsets into pitch classes, display names, note ranges, and active playback
//! indices.

use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, window};

/// Find a canvas by id and return it with a 2D rendering context.
pub(crate) fn canvas_context(
    canvas_id: &str,
    label: &str,
) -> Result<(HtmlCanvasElement, CanvasRenderingContext2d), JsValue> {
    let document = window()
        .ok_or_else(|| JsValue::from_str("window is not available"))?
        .document()
        .ok_or_else(|| JsValue::from_str("document is not available"))?;

    let canvas = document
        .get_element_by_id(canvas_id)
        .ok_or_else(|| JsValue::from_str(&format!("{label} was not found")))?
        .dyn_into::<HtmlCanvasElement>()?;

    let ctx = canvas
        .get_context("2d")?
        .ok_or_else(|| JsValue::from_str("2D canvas context is not available"))?
        .dyn_into::<CanvasRenderingContext2d>()?;

    Ok((canvas, ctx))
}

/// Convert normalized playback progress into a melody index.
pub(crate) fn active_note_index(note_count: usize, progress: f32) -> usize {
    if note_count == 0 {
        0
    } else {
        ((progress.clamp(0.0, 0.999_999) * note_count as f32).floor() as usize).min(note_count - 1)
    }
}

/// Return a non-empty pitch range suitable for graph scaling.
pub(crate) fn note_range(melody: &[i32]) -> (i32, i32) {
    let min_note = melody.iter().copied().min().unwrap_or(-12);
    let max_note = melody.iter().copied().max().unwrap_or(12);

    if min_note == max_note {
        (min_note - 1, max_note + 1)
    } else {
        (min_note, max_note)
    }
}

/// Convert a semitone offset from A4 into a MIDI-style pitch class.
pub(crate) fn pitch_class_from_semitone(semitone_from_a4: i32) -> i32 {
    (69 + semitone_from_a4).rem_euclid(12)
}

/// Whether the note would be drawn as a black piano key.
pub(crate) fn is_black_key(semitone_from_a4: i32) -> bool {
    matches!(
        pitch_class_from_semitone(semitone_from_a4),
        1 | 3 | 6 | 8 | 10
    )
}

/// Human-readable pitch-class name using sharps.
pub(crate) fn pitch_class_name(pitch_class: i32) -> &'static str {
    const NAMES: [&str; 12] = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];

    NAMES[pitch_class.rem_euclid(12) as usize]
}

/// Human-readable note name with octave, e.g. `A4` or `C#5`.
pub(crate) fn note_name(semitone_from_a4: i32) -> String {
    let midi_note = 69 + semitone_from_a4;
    let name = pitch_class_name(midi_note.rem_euclid(12));
    let octave = midi_note.div_euclid(12) - 1;

    format!("{name}{octave}")
}
