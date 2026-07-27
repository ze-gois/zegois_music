use std::f32::consts::TAU;

use wasm_bindgen::prelude::*;

const DEFAULT_BPM: f32 = 108.0;
const BEATS_PER_NOTE: f32 = 0.5;
const ATTACK_SECONDS: f32 = 0.01;
const RELEASE_SECONDS: f32 = 0.08;
const MASTER_GAIN: f32 = 0.22;

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
    ///
    /// JavaScript can place the returned Float32Array into an AudioBuffer.
    pub fn render_melody(&self, sample_rate: u32) -> Vec<f32> {
        render_melody(sample_rate, self.bpm)
    }

    pub fn frequency_for_semitone(&self, semitone_from_a4: i32) -> f32 {
        frequency_for_semitone(semitone_from_a4)
    }
}

#[wasm_bindgen]
pub fn render_melody(sample_rate: u32, bpm: f32) -> Vec<f32> {
    let sample_rate = sample_rate.max(8_000) as f32;
    let seconds_per_beat = 60.0 / bpm.clamp(30.0, 260.0);
    let note_seconds = seconds_per_beat * BEATS_PER_NOTE;
    let samples_per_note = (sample_rate * note_seconds).round() as usize;

    // Semitone offsets from A4. This is a simple A minor / C major flavored phrase.
    let melody = [
        0, 3, 7, 12, 10, 7, 3, 0, // A C E A G E C A
        -2, 2, 5, 10, 9, 5, 2, -2, // G B D G F# D B G
        -5, 0, 3, 7, 12, 7, 3, 0, // E A C E A E C A
        -7, -3, 0, 5, 3, 0, -3, -7, // D F A D C A F D
    ];

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

#[cfg(target_arch = "wasm32")]
pub use visualizer::WaveformVisualizer;

#[cfg(target_arch = "wasm32")]
mod visualizer {
    use wasm_bindgen::{JsCast, JsValue, prelude::*};
    use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, window};

    /// Canvas renderer for the generated audio waveform.
    ///
    /// JavaScript still owns browser audio timing, but all canvas drawing now
    /// happens in Rust/WASM.
    #[wasm_bindgen]
    pub struct WaveformVisualizer {
        canvas: HtmlCanvasElement,
        ctx: CanvasRenderingContext2d,
    }

    #[wasm_bindgen]
    impl WaveformVisualizer {
        #[wasm_bindgen(constructor)]
        pub fn new(canvas_id: &str) -> Result<WaveformVisualizer, JsValue> {
            let document = window()
                .ok_or_else(|| JsValue::from_str("window is not available"))?
                .document()
                .ok_or_else(|| JsValue::from_str("document is not available"))?;

            let canvas = document
                .get_element_by_id(canvas_id)
                .ok_or_else(|| JsValue::from_str("visualizer canvas was not found"))?
                .dyn_into::<HtmlCanvasElement>()?;

            let ctx = canvas
                .get_context("2d")?
                .ok_or_else(|| JsValue::from_str("2D canvas context is not available"))?
                .dyn_into::<CanvasRenderingContext2d>()?;

            Ok(WaveformVisualizer { canvas, ctx })
        }

        pub fn draw_idle(&self) -> Result<(), JsValue> {
            let width = self.width();
            let height = self.height();

            self.ctx.clear_rect(0.0, 0.0, width, height);
            self.ctx.set_fill_style_str("#10131f");
            self.ctx.fill_rect(0.0, 0.0, width, height);
            self.ctx.set_fill_style_str("#9aa4bf");
            self.ctx.set_font("20px system-ui, sans-serif");
            self.ctx.set_text_align("center");
            self.ctx.fill_text(
                "Rust is ready — press Play melody",
                width / 2.0,
                height / 2.0,
            )?;

            Ok(())
        }

        pub fn draw_waveform(&self, samples: &[f32], progress: f32) -> Result<(), JsValue> {
            if samples.is_empty() {
                return self.draw_idle();
            }

            let width = self.width();
            let height = self.height();
            let middle = height / 2.0;
            let played_x = width * progress.clamp(0.0, 1.0) as f64;

            self.ctx.clear_rect(0.0, 0.0, width, height);
            self.draw_background(width, height)?;
            self.draw_center_line(width, middle);
            self.draw_samples(samples, width, middle);
            self.draw_playhead(played_x, height);

            Ok(())
        }
    }

    impl WaveformVisualizer {
        fn width(&self) -> f64 {
            self.canvas.width() as f64
        }

        fn height(&self) -> f64 {
            self.canvas.height() as f64
        }

        fn draw_background(&self, width: f64, height: f64) -> Result<(), JsValue> {
            let gradient = self.ctx.create_linear_gradient(0.0, 0.0, width, height);
            gradient.add_color_stop(0.0, "#111827")?;
            gradient.add_color_stop(1.0, "#251438")?;

            self.ctx.set_fill_style_canvas_gradient(&gradient);
            self.ctx.fill_rect(0.0, 0.0, width, height);

            Ok(())
        }

        fn draw_center_line(&self, width: f64, middle: f64) {
            self.ctx.set_line_width(2.0);
            self.ctx.set_stroke_style_str("rgba(255, 255, 255, 0.12)");
            self.ctx.begin_path();
            self.ctx.move_to(0.0, middle);
            self.ctx.line_to(width, middle);
            self.ctx.stroke();
        }

        fn draw_samples(&self, samples: &[f32], width: f64, middle: f64) {
            let width_pixels = width.max(1.0).round() as usize;
            let step = (samples.len() / width_pixels).max(1);

            self.ctx.set_line_width(2.5);
            self.ctx.set_stroke_style_str("#77f7c5");
            self.ctx.begin_path();

            for x in 0..width_pixels {
                let start = x * step;
                if start >= samples.len() {
                    break;
                }

                let end = (start + step).min(samples.len());
                let (mut min, mut max) = (1.0_f32, -1.0_f32);

                for sample in &samples[start..end] {
                    min = min.min(*sample);
                    max = max.max(*sample);
                }

                let y1 = middle + min as f64 * middle * 0.82;
                let y2 = middle + max as f64 * middle * 0.82;
                let x = x as f64;

                self.ctx.move_to(x, y1);
                self.ctx.line_to(x, y2);
            }

            self.ctx.stroke();
        }

        fn draw_playhead(&self, played_x: f64, height: f64) {
            self.ctx.set_fill_style_str("rgba(255, 255, 255, 0.12)");
            self.ctx.fill_rect(0.0, 0.0, played_x, height);

            self.ctx.set_fill_style_str("#ffdf6b");
            self.ctx
                .fill_rect((played_x - 2.0).max(0.0), 0.0, 4.0, height);
        }
    }
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
