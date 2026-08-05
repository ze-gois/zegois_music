//! Clickable instrument visualizers for manual melody editing.
//!
//! The piano and guitar neck both draw familiar musical interfaces and convert
//! canvas-space click positions into semitone offsets from A4. The app then
//! auditions the note and applies the current edit mode.

use wasm_bindgen::JsValue;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

// use super::{canvas_context, is_black_key, note_name, pitch_class_from_semitone};

const PIANO_START: i32 = -21; // C3
const PIANO_END: i32 = 15; // C6

/// Clickable piano keyboard used to edit the selected melody instant.
pub struct PianoKeyboardVisualizer {
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
}

impl PianoKeyboardVisualizer {
    /// Create a piano visualizer bound to the canvas with `canvas_id`.
    pub fn new(canvas_id: &str) -> Result<PianoKeyboardVisualizer, JsValue> {
        let (canvas, ctx) =
            webspace::dom::canvas::canvas_context(canvas_id, "piano keyboard canvas")?;
        Ok(PianoKeyboardVisualizer { canvas, ctx })
    }

    /// Clone the underlying canvas so event handlers can attach listeners.
    pub fn canvas(&self) -> HtmlCanvasElement {
        self.canvas.clone()
    }

    /// Return the semitone offset under a canvas-space click, if any.
    pub fn note_at(&self, x: f64, y: f64) -> Option<i32> {
        let height = self.height();
        if !(0.0..=height).contains(&y) {
            return None;
        }

        let note_count = (PIANO_END - PIANO_START + 1) as usize;
        let key_width = self.width() / note_count as f64;
        let index = (x / key_width).floor() as i32;
        let note = PIANO_START + index;
        (PIANO_START..=PIANO_END).contains(&note).then_some(note)
    }

    /// Draw the keyboard and highlight `selected_note` when present.
    pub fn draw(&self, selected_note: Option<i32>) -> Result<(), JsValue> {
        let width = self.width();
        let height = self.height();
        let note_count = (PIANO_END - PIANO_START + 1) as usize;
        let key_width = width / note_count as f64;

        self.ctx.clear_rect(0.0, 0.0, width, height);
        self.ctx.set_fill_style_str("#0b1020");
        self.ctx.fill_rect(0.0, 0.0, width, height);

        for note in PIANO_START..=PIANO_END {
            if humans::audible::music::instrument::piano::is_black_key(note) {
                continue;
            }
            let x = (note - PIANO_START) as f64 * key_width;
            let is_selected = selected_note == Some(note);
            self.ctx
                .set_fill_style_str(if is_selected { "#ffdf6b" } else { "#f5f7fb" });
            self.ctx
                .fill_rect(x + 1.0, 12.0, key_width - 2.0, height - 24.0);
            self.ctx.set_stroke_style_str("rgba(7, 16, 20, 0.35)");
            self.ctx
                .stroke_rect(x + 1.0, 12.0, key_width - 2.0, height - 24.0);
        }

        for note in PIANO_START..=PIANO_END {
            if !humans::audible::music::instrument::piano::is_black_key(note) {
                continue;
            }
            let x = (note - PIANO_START) as f64 * key_width;
            let is_selected = selected_note == Some(note);
            self.ctx
                .set_fill_style_str(if is_selected { "#ffdf6b" } else { "#111827" });
            self.ctx
                .fill_rect(x + 1.0, 12.0, key_width - 2.0, height * 0.58);
        }

        self.ctx.set_font("13px system-ui, sans-serif");
        self.ctx.set_text_align("center");
        for note in PIANO_START..=PIANO_END {
            if humans::audible::music::note::pitch_class_from_semitone(note) != 0 {
                continue;
            }
            let x = (note - PIANO_START) as f64 * key_width + key_width / 2.0;
            self.ctx.set_fill_style_str("rgba(7, 16, 20, 0.70)");
            self.ctx.fill_text(
                &humans::audible::music::note::get_name_from_semitone(note),
                x,
                height - 18.0,
            )?;
        }

        self.ctx.set_text_align("left");
        self.ctx.set_fill_style_str("rgba(245, 247, 251, 0.72)");
        self.ctx.fill_text(
            "click a key to audition and apply the edit mode",
            16.0,
            24.0,
        )?;

        Ok(())
    }

    fn width(&self) -> f64 {
        self.canvas.width() as f64
    }

    fn height(&self) -> f64 {
        self.canvas.height() as f64
    }
}
