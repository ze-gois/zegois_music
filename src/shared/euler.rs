//! Euler/Tonnetz-inspired pitch-class graph.
//!
//! Nodes are arranged by the circle of fifths and linked by fifth/third
//! relationships. During playback the active pitch class is highlighted, and
//! during editing click hit-tests return a pitch class for the app to convert
//! into a concrete nearby note.

use wasm_bindgen::JsValue;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use webspace::dom::canvas::canvas_context;

use super::active_note_index;
use humans::audible::music::note::{pitch_class_from_semitone, pitch_class_name};

// use super::{active_note_index, canvas_context, pitch_class_from_semitone, pitch_class_name};

const CIRCLE_OF_FIFTHS: [(i32, &str); 12] = [
    (0, "C"),
    (7, "G"),
    (2, "D"),
    (9, "A"),
    (4, "E"),
    (11, "B"),
    (6, "F#"),
    (1, "C#"),
    (8, "G#"),
    (3, "D#"),
    (10, "A#"),
    (5, "F"),
];

/// Draws and hit-tests the circular pitch-class relationship graph.
pub struct EulerGraphVisualizer {
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
}

impl EulerGraphVisualizer {
    /// Create a graph visualizer bound to the canvas with `canvas_id`.
    pub fn new(canvas_id: &str) -> Result<EulerGraphVisualizer, JsValue> {
        let (canvas, ctx) = canvas_context(canvas_id, "Euler/Tonnetz graph canvas")?;
        Ok(EulerGraphVisualizer { canvas, ctx })
    }

    /// Draw graph relationships, melody trace, and active pitch class.
    pub fn draw(&self, melody: &[i32], progress: f32) -> Result<(), JsValue> {
        let width = self.width();
        let height = self.height();
        let active_note = active_note_index(melody.len(), progress);
        let active_pitch_class = melody
            .get(active_note)
            .copied()
            .map(pitch_class_from_semitone);

        self.ctx.clear_rect(0.0, 0.0, width, height);
        self.draw_background(width, height)?;
        self.draw_relationship_links(width, height);
        self.draw_melody_trace(melody, width, height, active_note);
        self.draw_nodes(width, height, active_pitch_class)?;
        self.draw_title(melody, width, height, active_note)?;

        Ok(())
    }

    fn width(&self) -> f64 {
        self.canvas.width() as f64
    }

    /// Clone the underlying canvas so event handlers can attach listeners.
    pub fn canvas(&self) -> HtmlCanvasElement {
        self.canvas.clone()
    }

    /// Return the pitch class under a canvas-space click, if any.
    pub fn pitch_class_at(&self, x: f64, y: f64) -> Option<i32> {
        let width = self.width();
        let height = self.height();
        CIRCLE_OF_FIFTHS.iter().find_map(|(pitch_class, _)| {
            let (node_x, node_y) = self.position_for_pitch_class(*pitch_class, width, height);
            let distance = ((node_x - x).powi(2) + (node_y - y).powi(2)).sqrt();
            (distance <= 24.0).then_some(*pitch_class)
        })
    }

    fn height(&self) -> f64 {
        self.canvas.height() as f64
    }

    fn center(&self, width: f64, height: f64) -> (f64, f64) {
        (width / 2.0, height / 2.0 + 4.0)
    }

    fn radius(&self, width: f64, height: f64) -> f64 {
        (width.min(height) * 0.36).max(80.0)
    }

    fn position_for_pitch_class(&self, pitch_class: i32, width: f64, height: f64) -> (f64, f64) {
        let index = CIRCLE_OF_FIFTHS
            .iter()
            .position(|(pc, _)| *pc == pitch_class.rem_euclid(12))
            .unwrap_or(0);
        let angle = -std::f64::consts::FRAC_PI_2
            + index as f64 / CIRCLE_OF_FIFTHS.len() as f64 * std::f64::consts::TAU;
        let (cx, cy) = self.center(width, height);
        let radius = self.radius(width, height);

        (cx + radius * angle.cos(), cy + radius * angle.sin())
    }

    fn draw_background(&self, width: f64, height: f64) -> Result<(), JsValue> {
        let gradient = self.ctx.create_radial_gradient(
            width * 0.5,
            height * 0.44,
            20.0,
            width * 0.5,
            height * 0.5,
            width.max(height) * 0.7,
        )?;
        gradient.add_color_stop(0.0, "#172033")?;
        gradient.add_color_stop(1.0, "#0b1020")?;

        self.ctx.set_fill_style_canvas_gradient(&gradient);
        self.ctx.fill_rect(0.0, 0.0, width, height);

        Ok(())
    }

    fn draw_relationship_links(&self, width: f64, height: f64) {
        self.draw_fifth_cycle(width, height);
        self.draw_interval_links(width, height, 4, "rgba(255, 223, 107, 0.20)", 1.4);
        self.draw_interval_links(width, height, 3, "rgba(168, 85, 247, 0.20)", 1.2);
    }

    fn draw_fifth_cycle(&self, width: f64, height: f64) {
        self.ctx.set_line_width(2.5);
        self.ctx.set_stroke_style_str("rgba(119, 247, 197, 0.34)");

        for index in 0..CIRCLE_OF_FIFTHS.len() {
            let from = CIRCLE_OF_FIFTHS[index].0;
            let to = CIRCLE_OF_FIFTHS[(index + 1) % CIRCLE_OF_FIFTHS.len()].0;
            self.draw_line_between_pitch_classes(from, to, width, height);
        }
    }

    fn draw_interval_links(
        &self,
        width: f64,
        height: f64,
        interval: i32,
        color: &str,
        line_width: f64,
    ) {
        self.ctx.set_line_width(line_width);
        self.ctx.set_stroke_style_str(color);

        for pitch_class in 0..12 {
            self.draw_line_between_pitch_classes(
                pitch_class,
                (pitch_class + interval).rem_euclid(12),
                width,
                height,
            );
        }
    }

    fn draw_melody_trace(&self, melody: &[i32], width: f64, height: f64, active_note: usize) {
        if melody.len() <= 1 {
            return;
        }

        self.ctx.set_line_width(4.0);
        self.ctx.set_stroke_style_str("rgba(255, 223, 107, 0.58)");
        self.ctx.begin_path();

        for note_index in 0..=active_note.min(melody.len() - 1) {
            let pitch_class = pitch_class_from_semitone(melody[note_index]);
            let (x, y) = self.position_for_pitch_class(pitch_class, width, height);

            if note_index == 0 {
                self.ctx.move_to(x, y);
            } else {
                self.ctx.line_to(x, y);
            }
        }

        self.ctx.stroke();
    }

    fn draw_line_between_pitch_classes(&self, from: i32, to: i32, width: f64, height: f64) {
        let (x1, y1) = self.position_for_pitch_class(from, width, height);
        let (x2, y2) = self.position_for_pitch_class(to, width, height);

        self.ctx.begin_path();
        self.ctx.move_to(x1, y1);
        self.ctx.line_to(x2, y2);
        self.ctx.stroke();
    }

    fn draw_nodes(
        &self,
        width: f64,
        height: f64,
        active_pitch_class: Option<i32>,
    ) -> Result<(), JsValue> {
        for (pitch_class, name) in CIRCLE_OF_FIFTHS {
            let (x, y) = self.position_for_pitch_class(pitch_class, width, height);
            let is_active = active_pitch_class == Some(pitch_class);
            let radius = if is_active { 20.0 } else { 15.0 };

            self.ctx.begin_path();
            self.ctx
                .set_fill_style_str(if is_active { "#ffdf6b" } else { "#101827" });
            self.ctx.set_stroke_style_str(if is_active {
                "rgba(255, 255, 255, 0.95)"
            } else {
                "rgba(119, 247, 197, 0.82)"
            });
            self.ctx.set_line_width(if is_active { 4.0 } else { 2.0 });
            let _ = self.ctx.arc(x, y, radius, 0.0, std::f64::consts::TAU);
            self.ctx.fill();
            self.ctx.stroke();

            self.ctx.set_font(if is_active {
                "bold 16px system-ui, sans-serif"
            } else {
                "14px system-ui, sans-serif"
            });
            self.ctx.set_text_align("center");
            self.ctx
                .set_fill_style_str(if is_active { "#071014" } else { "#f5f7fb" });
            self.ctx.fill_text(name, x, y + 5.0)?;
        }

        Ok(())
    }

    fn draw_title(
        &self,
        melody: &[i32],
        width: f64,
        height: f64,
        active_note: usize,
    ) -> Result<(), JsValue> {
        let pitch_class = melody
            .get(active_note)
            .copied()
            .map(pitch_class_from_semitone);

        self.ctx.set_font("14px system-ui, sans-serif");
        self.ctx.set_text_align("left");
        self.ctx.set_fill_style_str("rgba(245, 247, 251, 0.72)");
        self.ctx
            .fill_text("circle of fifths with Tonnetz-like third links", 24.0, 28.0)?;

        self.ctx.set_fill_style_str("#ffdf6b");
        let active_label = pitch_class
            .map(pitch_class_name)
            .map(|name| format!("active pitch class: {name}"))
            .unwrap_or_else(|| "click a node to begin composing".to_string());
        self.ctx.fill_text(&active_label, 24.0, height - 18.0)?;

        self.ctx.set_text_align("right");
        self.ctx.set_fill_style_str("rgba(245, 247, 251, 0.56)");
        self.ctx.fill_text(
            "green: fifths · gold: major thirds · purple: minor thirds",
            width - 24.0,
            height - 18.0,
        )?;

        Ok(())
    }
}
