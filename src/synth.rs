use std::f32::consts::TAU;

use wasm_bindgen::prelude::*;

pub(crate) const DEFAULT_BPM: f32 = 108.0;
const BEATS_PER_NOTE: f32 = 0.5;
const ATTACK_SECONDS: f32 = 0.01;
const RELEASE_SECONDS: f32 = 0.08;
const MASTER_GAIN: f32 = 0.22;

// Semitone offsets from A4. This is a simple A minor / C major flavored phrase.
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
    #[wasm_bindgen(constructor)]
    pub fn new() -> Synth {
        Synth { bpm: DEFAULT_BPM }
    }

    pub fn bpm(&self) -> f32 {
        self.bpm
    }

    pub fn set_bpm(&mut self, bpm: f32) {
        self.bpm = bpm.clamp(30.0, 260.0);
    }

    /// Render a short looping-ish melody as mono f32 PCM samples in [-1.0, 1.0].
    pub fn render_melody(&self, sample_rate: u32) -> Vec<f32> {
        render_melody(sample_rate, self.bpm)
    }

    pub fn frequency_for_semitone(&self, semitone_from_a4: i32) -> f32 {
        frequency_for_semitone(semitone_from_a4)
    }
}

impl Default for Synth {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
pub fn render_melody(sample_rate: u32, bpm: f32) -> Vec<f32> {
    render_notes(sample_rate, bpm, &DEFAULT_MELODY)
}

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

#[wasm_bindgen]
pub fn frequency_for_semitone(semitone_from_a4: i32) -> f32 {
    440.0 * 2.0_f32.powf(semitone_from_a4 as f32 / 12.0)
}

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
