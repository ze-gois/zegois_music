use wasm_bindgen::JsValue;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use humans::audible::music::note;
use webspace::dom::canvas::canvas_context;

const GUITAR_STRINGS: [i32; 6] = [-5, -10, -14, -19, -24, -29]; // E4 B3 G3 D3 A2 E2
const GUITAR_FRETS: usize = 17;

/// Clickable guitar neck used to edit the selected melody instant.
pub struct GuitarNeckVisualizer {
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
}

impl GuitarNeckVisualizer {
    /// Create a guitar-neck visualizer bound to the canvas with `canvas_id`.
    pub fn new(canvas_id: &str) -> Result<GuitarNeckVisualizer, JsValue> {
        let (canvas, ctx) = canvas_context(canvas_id, "guitar neck canvas")?;
        Ok(GuitarNeckVisualizer { canvas, ctx })
    }

    /// Clone the underlying canvas so event handlers can attach listeners.
    pub fn canvas(&self) -> HtmlCanvasElement {
        self.canvas.clone()
    }

    /// Return the semitone offset under a canvas-space click, if any.
    pub fn note_at(&self, x: f64, y: f64) -> Option<i32> {
        let width = self.width();
        let height = self.height();
        let left = 72.0;
        let right = width - 24.0;
        let top = 44.0;
        let bottom = height - 44.0;

        if x < left || x > right || y < top - 16.0 || y > bottom + 16.0 {
            return None;
        }

        let string_gap = (bottom - top) / (GUITAR_STRINGS.len() - 1) as f64;
        let string_index = ((y - top) / string_gap).round() as isize;
        if string_index < 0 || string_index as usize >= GUITAR_STRINGS.len() {
            return None;
        }

        let fret_width = (right - left) / GUITAR_FRETS as f64;
        let fret = ((x - left) / fret_width)
            .floor()
            .clamp(0.0, GUITAR_FRETS as f64) as i32;
        Some(GUITAR_STRINGS[string_index as usize] + fret)
    }

    /// Draw the guitar neck and highlight all matching selected notes.
    pub fn draw(&self, selected_note: Option<i32>) -> Result<(), JsValue> {
        let width = self.width();
        let height = self.height();
        let left = 72.0;
        let right = width - 24.0;
        let top = 44.0;
        let bottom = height - 44.0;
        let fret_width = (right - left) / GUITAR_FRETS as f64;
        let string_gap = (bottom - top) / (GUITAR_STRINGS.len() - 1) as f64;

        self.ctx.clear_rect(0.0, 0.0, width, height);
        self.ctx.set_fill_style_str("#16100b");
        self.ctx.fill_rect(0.0, 0.0, width, height);
        self.ctx.set_fill_style_str("#3b2615");
        self.ctx
            .fill_rect(left, top - 20.0, right - left, bottom - top + 40.0);

        for fret in 0..=GUITAR_FRETS {
            let x = left + fret as f64 * fret_width;
            self.ctx.set_stroke_style_str(if fret == 0 {
                "#f5f7fb"
            } else {
                "rgba(245, 247, 251, 0.45)"
            });
            self.ctx.set_line_width(if fret == 0 { 5.0 } else { 2.0 });
            self.ctx.begin_path();
            self.ctx.move_to(x, top - 20.0);
            self.ctx.line_to(x, bottom + 20.0);
            self.ctx.stroke();
        }

        self.ctx.set_stroke_style_str("rgba(245, 247, 251, 0.72)");
        for string_index in 0..GUITAR_STRINGS.len() {
            let y = top + string_index as f64 * string_gap;
            self.ctx.set_line_width(1.5 + string_index as f64 * 0.35);
            self.ctx.begin_path();
            self.ctx.move_to(left, y);
            self.ctx.line_to(right, y);
            self.ctx.stroke();

            self.ctx.set_font("13px system-ui, sans-serif");
            self.ctx.set_text_align("right");
            self.ctx.set_fill_style_str("rgba(245, 247, 251, 0.72)");
            self.ctx.fill_text(
                &note::get_name_from_semitone(GUITAR_STRINGS[string_index]),
                left - 12.0,
                y + 4.0,
            )?;
        }

        for string_index in 0..GUITAR_STRINGS.len() {
            for fret in 0..=GUITAR_FRETS {
                let note = GUITAR_STRINGS[string_index] + fret as i32;
                if selected_note != Some(note) {
                    continue;
                }
                let x = left + fret as f64 * fret_width + fret_width / 2.0;
                let y = top + string_index as f64 * string_gap;
                self.ctx.set_fill_style_str("#ffdf6b");
                self.ctx.begin_path();
                let _ = self.ctx.arc(x, y, 10.0, 0.0, std::f64::consts::TAU);
                self.ctx.fill();
            }
        }

        for fret in [3, 5, 7, 9, 12, 15] {
            if fret > GUITAR_FRETS {
                continue;
            }
            let x = left + fret as f64 * fret_width - fret_width / 2.0;
            self.ctx.set_fill_style_str("rgba(245, 247, 251, 0.28)");
            self.ctx.begin_path();
            let _ = self
                .ctx
                .arc(x, (top + bottom) / 2.0, 5.0, 0.0, std::f64::consts::TAU);
            self.ctx.fill();
        }

        self.ctx.set_font("13px system-ui, sans-serif");
        self.ctx.set_text_align("left");
        self.ctx.set_fill_style_str("rgba(245, 247, 251, 0.72)");
        self.ctx.fill_text(
            "standard tuning · click a string/fret to audition and apply the edit mode",
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
