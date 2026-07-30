//! Waveform canvas visualizer.
//!
//! The waveform visualizer is the one visualization exported directly to
//! JavaScript for the original MVP API. The Rust-owned app also uses it to draw
//! synthesized samples and a playback playhead.

use wasm_bindgen::{JsCast, JsValue, prelude::*};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, window};

/// Draws PCM samples and playback progress onto a canvas.
#[wasm_bindgen]
pub struct WaveformVisualizer {
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
}

#[wasm_bindgen]
impl WaveformVisualizer {
    /// Create a visualizer bound to the canvas with `canvas_id`.
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

    /// Draw the idle state shown before audio has been rendered.
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
            "Rust UI is ready — press Play melody",
            width / 2.0,
            height / 2.0,
        )?;

        Ok(())
    }

    /// Draw `samples` with `progress` in the range `0.0..=1.0`.
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
