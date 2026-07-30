//! Canvas visualizers and hit-testing for the music UI.
//!
//! Each visualizer owns one `<canvas>` and its 2D context. Drawing code lives in
//! Rust so playback state, melody edits, and visual feedback share the same data
//! model. Instrument visualizers also expose hit-testing methods that convert
//! click positions into semitone offsets.

mod common;
mod euler;
mod instruments;
mod note_graph;
mod waveform;

pub(crate) use common::{
    active_note_index, canvas_context, is_black_key, note_name, note_range,
    pitch_class_from_semitone, pitch_class_name,
};
pub(crate) use euler::EulerGraphVisualizer;
pub(crate) use instruments::{GuitarNeckVisualizer, PianoKeyboardVisualizer};
pub(crate) use note_graph::NoteGraphVisualizer;
pub use waveform::WaveformVisualizer;
