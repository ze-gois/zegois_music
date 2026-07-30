//! Audio synthesis primitives.
//!
//! This module is intentionally small and deterministic: it turns semitone
//! offsets from A4 into equal-temperament frequencies, then renders a melody as
//! mono `f32` PCM samples. The WASM app copies those samples into a Web Audio
//! `AudioBuffer`, while native tests use the same code path.

use std::f32::consts::TAU;

use wasm_bindgen::prelude::*;

pub(crate) const DEFAULT_BPM: f32 = 108.0;
const BEATS_PER_NOTE: f32 = 0.5;
const ATTACK_SECONDS: f32 = 0.01;
const RELEASE_SECONDS: f32 = 0.08;
const MASTER_GAIN: f32 = 0.22;

/// Built-in 32-step phrase used when the browser UI starts or resets.
///
/// Each value is a semitone offset from A4. The phrase has an A minor / C major
/// flavor and gives every visualizer enough motion to be interesting before the
/// user edits it.
pub(crate) const DEFAULT_MELODY: [i32; 32] = [
    0, 3, 7, 12, 10, 7, 3, 0, // A C E A G E C A
    -2, 2, 5, 10, 9, 5, 2, -2, // G B D G F# D B G
    -5, 0, 3, 7, 12, 7, 3, 0, // E A C E A E C A
    -7, -3, 0, 5, 3, 0, -3, -7, // D F A D C A F D
];

/// A tiny browser-friendly synth that renders a fixed melody into samples.
///
/// Notes are represented as semitone offsets from A4 = 440Hz.
#[wasm_bindgen]
pub struct Synth {
    bpm: f32,
}

#[wasm_bindgen]
impl Synth {
    /// Create a synth with the default tempo.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Synth {
        Synth { bpm: DEFAULT_BPM }
    }

    /// Current tempo in beats per minute.
    pub fn bpm(&self) -> f32 {
        self.bpm
    }

    /// Set the tempo, clamped to a musically useful browser-demo range.
    pub fn set_bpm(&mut self, bpm: f32) {
        self.bpm = bpm.clamp(30.0, 260.0);
    }

    /// Render a short looping-ish melody as mono f32 PCM samples in [-1.0, 1.0].
    pub fn render_melody(&self, sample_rate: u32) -> Vec<f32> {
        render_melody(sample_rate, self.bpm)
    }

    /// Convert a semitone offset from A4 into hertz.
    pub fn frequency_for_semitone(&self, semitone_from_a4: i32) -> f32 {
        frequency_for_semitone(semitone_from_a4)
    }
}

impl Default for Synth {
    fn default() -> Self {
        Self::new()
    }
}

/// Render the built-in melody as mono `f32` PCM samples.
///
/// This free function is exported to JavaScript as a simple MVP API. The app
/// state uses the same rendering path internally when the user edits the melody.
#[wasm_bindgen]
pub fn render_melody(sample_rate: u32, bpm: f32) -> Vec<f32> {
    render_notes(sample_rate, bpm, &DEFAULT_MELODY)
}

/// Render an arbitrary melody as mono `f32` PCM samples.
///
/// `melody` is a list of semitone offsets from A4. A small attack/release
/// envelope avoids clicks, and a quiet first overtone/vibrato gives the sine
/// wave a little life while keeping the MVP easy to understand.
pub(crate) fn render_notes(sample_rate: u32, bpm: f32, melody: &[i32]) -> Vec<f32> {
    let sample_rate = sample_rate.max(8_000) as f32;
    let seconds_per_beat = 60.0 / bpm.clamp(30.0, 260.0);
    let note_seconds = seconds_per_beat * BEATS_PER_NOTE;
    let samples_per_note = (sample_rate * note_seconds).round() as usize;

    let mut samples = Vec::with_capacity(samples_per_note * melody.len());

    for (note_index, semitone) in melody.iter().enumerate() {
        let frequency = frequency_for_semitone(*semitone);
        let harmonic = frequency * 2.0;

        for i in 0..samples_per_note {
            let t = i as f32 / sample_rate;
            let absolute_t = (note_index * samples_per_note + i) as f32 / sample_rate;

            let envelope = envelope(i, samples_per_note, sample_rate);
            let vibrato = 1.0 + 0.004 * (TAU * 5.0 * absolute_t).sin();

            let fundamental = (TAU * frequency * vibrato * t).sin();
            let overtone = 0.35 * (TAU * harmonic * vibrato * t).sin();
            let sample = (fundamental + overtone) * envelope * MASTER_GAIN;

            samples.push(sample.clamp(-1.0, 1.0));
        }
    }

    samples
}

/// Convert semitones from A4 to equal-temperament frequency in hertz.
///
/// The formula is `440 * 2^(n / 12)`, where `n` is the semitone offset from A4.
#[wasm_bindgen]
pub fn frequency_for_semitone(semitone_from_a4: i32) -> f32 {
    440.0 * 2.0_f32.powf(semitone_from_a4 as f32 / 12.0)
}

/// Simple linear attack/release envelope for a single note.
fn envelope(sample_index: usize, total_samples: usize, sample_rate: f32) -> f32 {
    let attack_samples = (ATTACK_SECONDS * sample_rate) as usize;
    let release_samples = (RELEASE_SECONDS * sample_rate) as usize;
    let release_start = total_samples.saturating_sub(release_samples);

    if attack_samples > 0 && sample_index < attack_samples {
        sample_index as f32 / attack_samples as f32
    } else if release_samples > 0 && sample_index > release_start {
        let samples_left = total_samples.saturating_sub(sample_index);
        samples_left as f32 / release_samples as f32
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computes_equal_temperament_frequencies() {
        assert!((frequency_for_semitone(0) - 440.0).abs() < 0.001);
        assert!((frequency_for_semitone(12) - 880.0).abs() < 0.001);
        assert!((frequency_for_semitone(-12) - 220.0).abs() < 0.001);
    }

    #[test]
    fn renders_samples_in_audio_range() {
        let samples = render_melody(44_100, DEFAULT_BPM);

        assert!(!samples.is_empty());
        assert!(samples.iter().all(|sample| (-1.0..=1.0).contains(sample)));
    }
}
