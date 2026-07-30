//! Pitch-over-time note graph.
//!
//! This visualizer maps each melody instant to a point on a time/pitch graph,
//! connects the points, and overlays a playhead plus active-note labels.

use wasm_bindgen::JsValue;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use super::{active_note_index, canvas_context, note_name, note_range};

/// Draws the melody as pitch over time.
pub struct NoteGraphVisualizer {
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
}

impl NoteGraphVisualizer {
    /// Create a note graph bound to the canvas with `canvas_id`.
    pub fn new(canvas_id: &str) -> Result<NoteGraphVisualizer, JsValue> {
        let (canvas, ctx) = canvas_context(canvas_id, "note graph canvas")?;

        Ok(NoteGraphVisualizer { canvas, ctx })
    }

    /// Draw the melody and active playback position.
    pub fn draw(&self, melody: &[i32], progress: f32) -> Result<(), JsValue> {
        let width = self.width();
        let height = self.height();
        let progress = progress.clamp(0.0, 1.0);
        let played_x = width * progress as f64;
        let active_note = active_note_index(melody.len(), progress);
        let (min_note, max_note) = note_range(melody);

        self.ctx.clear_rect(0.0, 0.0, width, height);
        self.draw_background(width, height)?;
        self.draw_grid(melody.len(), min_note, max_note, width, height);
        self.draw_note_path(melody, min_note, max_note, width, height);
        self.draw_note_nodes(melody, min_note, max_note, width, height, active_note);
        self.draw_playhead(played_x, height);
        self.draw_labels(melody, min_note, max_note, width, height, active_note)?;

        Ok(())
    }

    fn width(&self) -> f64 {
        self.canvas.width() as f64
    }

    fn height(&self) -> f64 {
        self.canvas.height() as f64
    }

    fn x_for_note(&self, note_index: usize, note_count: usize, width: f64) -> f64 {
        if note_count <= 1 {
            width / 2.0
        } else {
            let padding = 52.0;
            let usable_width = (width - padding * 2.0).max(1.0);
            padding + usable_width * note_index as f64 / (note_count - 1) as f64
        }
    }

    fn y_for_note(&self, semitone: i32, min_note: i32, max_note: i32, height: f64) -> f64 {
        let padding = 34.0;
        let usable_height = (height - padding * 2.0).max(1.0);
        let note_range = (max_note - min_note).max(1) as f64;
        let normalized = (semitone - min_note) as f64 / note_range;

        padding + usable_height * (1.0 - normalized)
    }

    fn draw_background(&self, width: f64, height: f64) -> Result<(), JsValue> {
        let gradient = self.ctx.create_linear_gradient(0.0, 0.0, width, height);
        gradient.add_color_stop(0.0, "#0f172a")?;
        gradient.add_color_stop(1.0, "#1e1535")?;

        self.ctx.set_fill_style_canvas_gradient(&gradient);
        self.ctx.fill_rect(0.0, 0.0, width, height);

        Ok(())
    }

    fn draw_grid(&self, note_count: usize, min_note: i32, max_note: i32, width: f64, height: f64) {
        self.ctx.set_line_width(1.0);
        self.ctx.set_stroke_style_str("rgba(255, 255, 255, 0.10)");

        for semitone in min_note..=max_note {
            let y = self.y_for_note(semitone, min_note, max_note, height);
            self.ctx.begin_path();
            self.ctx.move_to(42.0, y);
            self.ctx.line_to(width - 18.0, y);
            self.ctx.stroke();
        }

        self.ctx.set_stroke_style_str("rgba(119, 247, 197, 0.12)");
        for note_index in 0..note_count {
            let x = self.x_for_note(note_index, note_count, width);
            self.ctx.begin_path();
            self.ctx.move_to(x, 24.0);
            self.ctx.line_to(x, height - 24.0);
            self.ctx.stroke();
        }
    }

    fn draw_note_path(
        &self,
        melody: &[i32],
        min_note: i32,
        max_note: i32,
        width: f64,
        height: f64,
    ) {
        if melody.is_empty() {
            return;
        }

        self.ctx.set_line_width(3.0);
        self.ctx.set_stroke_style_str("#77f7c5");
        self.ctx.begin_path();

        for (note_index, semitone) in melody.iter().enumerate() {
            let x = self.x_for_note(note_index, melody.len(), width);
            let y = self.y_for_note(*semitone, min_note, max_note, height);

            if note_index == 0 {
                self.ctx.move_to(x, y);
            } else {
                self.ctx.line_to(x, y);
            }
        }

        self.ctx.stroke();
    }

    fn draw_note_nodes(
        &self,
        melody: &[i32],
        min_note: i32,
        max_note: i32,
        width: f64,
        height: f64,
        active_note: usize,
    ) {
        for (note_index, semitone) in melody.iter().enumerate() {
            let x = self.x_for_note(note_index, melody.len(), width);
            let y = self.y_for_note(*semitone, min_note, max_note, height);
            let is_active = note_index == active_note;
            let radius = if is_active { 8.0 } else { 4.5 };

            self.ctx.begin_path();
            self.ctx
                .set_fill_style_str(if is_active { "#ffdf6b" } else { "#77f7c5" });
            let _ = self.ctx.arc(x, y, radius, 0.0, std::f64::consts::TAU);
            self.ctx.fill();
        }
    }

    fn draw_playhead(&self, played_x: f64, height: f64) {
        self.ctx.set_fill_style_str("rgba(255, 255, 255, 0.10)");
        self.ctx.fill_rect(0.0, 0.0, played_x, height);

        self.ctx.set_fill_style_str("#ffdf6b");
        self.ctx
            .fill_rect((played_x - 2.0).max(0.0), 0.0, 4.0, height);
    }

    fn draw_labels(
        &self,
        melody: &[i32],
        min_note: i32,
        max_note: i32,
        width: f64,
        height: f64,
        active_note: usize,
    ) -> Result<(), JsValue> {
        self.ctx.set_font("14px system-ui, sans-serif");
        self.ctx.set_fill_style_str("rgba(245, 247, 251, 0.72)");
        self.ctx.set_text_align("right");

        for semitone in min_note..=max_note {
            let y = self.y_for_note(semitone, min_note, max_note, height) + 4.0;
            self.ctx.fill_text(&note_name(semitone), 34.0, y)?;
        }

        let Some(semitone) = melody.get(active_note).copied() else {
            self.ctx.set_text_align("left");
            self.ctx.set_fill_style_str("#ffdf6b");
            self.ctx.fill_text(
                "click Euler/Tonnetz nodes to add notes",
                52.0,
                height - 12.0,
            )?;
            return Ok(());
        };
        let frequency = crate::frequency_for_semitone(semitone);
        self.ctx.set_text_align("left");
        self.ctx.set_fill_style_str("#ffdf6b");
        self.ctx.fill_text(
            &format!("{} · {:.1} Hz", note_name(semitone), frequency),
            52.0,
            height - 12.0,
        )?;

        self.ctx.set_text_align("right");
        self.ctx.set_fill_style_str("rgba(245, 247, 251, 0.56)");
        self.ctx
            .fill_text("pitch over time", width - 18.0, height - 12.0)?;

        Ok(())
    }
}
