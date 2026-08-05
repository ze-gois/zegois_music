pub mod content;
pub mod euler;
pub mod graph;
pub mod guitar;
pub mod piano;

/// Convert normalized playback progress into a melody index.
pub(crate) fn active_note_index(note_count: usize, progress: f32) -> usize {
    if note_count == 0 {
        0
    } else {
        ((progress.clamp(0.0, 0.999_999) * note_count as f32).floor() as usize).min(note_count - 1)
    }
}
