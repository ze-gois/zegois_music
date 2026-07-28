use std::f32::consts::TAU;

use wasm_bindgen::prelude::*;

const DEFAULT_BPM: f32 = 108.0;
const BEATS_PER_NOTE: f32 = 0.5;
const ATTACK_SECONDS: f32 = 0.01;
const RELEASE_SECONDS: f32 = 0.08;
const MASTER_GAIN: f32 = 0.22;

// Semitone offsets from A4. This is a simple A minor / C major flavored phrase.
const MELODY: [i32; 32] = [
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
    let sample_rate = sample_rate.max(8_000) as f32;
    let seconds_per_beat = 60.0 / bpm.clamp(30.0, 260.0);
    let note_seconds = seconds_per_beat * BEATS_PER_NOTE;
    let samples_per_note = (sample_rate * note_seconds).round() as usize;

    let mut samples = Vec::with_capacity(samples_per_note * MELODY.len());

    for (note_index, semitone) in MELODY.iter().enumerate() {
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
pub use app::start_app;
#[cfg(target_arch = "wasm32")]
pub use visualizer::WaveformVisualizer;

#[cfg(target_arch = "wasm32")]
mod app {
    use std::{cell::RefCell, rc::Rc};

    use wasm_bindgen::{JsCast, JsValue, closure::Closure, prelude::*};
    use web_sys::{
        AudioBufferSourceNode, AudioContext, AudioScheduledSourceNode, Document, Element,
        HtmlButtonElement, HtmlInputElement, Window, window,
    };

    use super::{
        DEFAULT_BPM, Synth,
        visualizer::{EulerGraphVisualizer, NoteGraphVisualizer, WaveformVisualizer},
    };

    const APP_HTML: &str = r#"
<main>
  <section class="hero">
    <p class="eyebrow">Rust + WebAssembly + Web Audio</p>
    <h1>Music</h1>
    <p>
      A tiny synthesized melody rendered from sine waves in Rust, played in
      the browser, visualized on canvas, and now controlled by a Rust-built UI.
    </p>
  </section>

  <section class="panel">
    <div class="controls">
      <button id="play">Play melody</button>
      <button id="stop" disabled>Stop!</button>
      <label>
        BPM
        <input id="bpm" type="range" min="60" max="180" value="108" />
        <span id="bpmValue">108</span>
      </label>
    </div>

    <div class="visual-stack">
      <div>
        <h2>Waveform</h2>
        <canvas id="visualizer" width="960" height="320" aria-label="Waveform visualizer"></canvas>
      </div>
      <div>
        <h2>Note graph</h2>
        <canvas id="noteGraph" width="960" height="260" aria-label="Pitch over time note graph"></canvas>
      </div>
      <div>
        <h2>Euler/Tonnetz graph</h2>
        <canvas id="eulerGraph" width="960" height="360" aria-label="Circular linked graph of pitch-class relationships"></canvas>
      </div>
    </div>
    <p id="status" class="status">Ready. Press Play melody to synthesize sound from Rust.</p>
  </section>

  <section class="notes">
    <h2>What is happening?</h2>
    <ul>
      <li>Rust creates this UI and binds the controls.</li>
      <li>Rust computes note frequencies with <code>440 * 2^(n / 12)</code>.</li>
      <li>Rust renders mono <code>f32</code> PCM samples for a small melody.</li>
      <li>Rust sends those samples to a Web Audio <code>AudioBuffer</code>.</li>
      <li>Rust draws the same samples as a waveform on the canvas.</li>
      <li>Rust also graphs the melody as pitch over time and highlights the active note.</li>
      <li>Rust draws an Euler/Tonnetz-inspired graph linking pitch classes by fifths and thirds.</li>
    </ul>
  </section>
</main>
"#;

    #[wasm_bindgen]
    pub fn start_app() -> Result<(), JsValue> {
        let window = window().ok_or_else(|| JsValue::from_str("window is not available"))?;
        let document = window
            .document()
            .ok_or_else(|| JsValue::from_str("document is not available"))?;

        let root = document
            .get_element_by_id("app")
            .ok_or_else(|| JsValue::from_str("#app root element was not found"))?;
        root.set_inner_html(APP_HTML);

        let state = Rc::new(RefCell::new(AppState::new(&document)?));
        {
            let state = state.borrow();
            state.visualizer.draw_idle()?;
            state.note_graph.draw(0.0)?;
            state.euler_graph.draw(0.0)?;
        }

        let animation = create_animation_loop(&window, Rc::clone(&state));
        bind_bpm_input(Rc::clone(&state))?;
        bind_play_button(&window, Rc::clone(&state), Rc::clone(&animation))?;
        bind_stop_button(&window, Rc::clone(&state))?;

        Ok(())
    }

    struct AppState {
        synth: Synth,
        visualizer: WaveformVisualizer,
        note_graph: NoteGraphVisualizer,
        euler_graph: EulerGraphVisualizer,
        audio_context: Option<AudioContext>,
        source: Option<AudioBufferSourceNode>,
        samples: Vec<f32>,
        started_at: f64,
        animation_frame: Option<i32>,
        play_button: HtmlButtonElement,
        stop_button: HtmlButtonElement,
        bpm_input: HtmlInputElement,
        bpm_value: Element,
        status: Element,
    }

    impl AppState {
        fn new(document: &Document) -> Result<Self, JsValue> {
            Ok(Self {
                synth: Synth::new(),
                visualizer: WaveformVisualizer::new("visualizer")?,
                note_graph: NoteGraphVisualizer::new("noteGraph")?,
                euler_graph: EulerGraphVisualizer::new("eulerGraph")?,
                audio_context: None,
                source: None,
                samples: Vec::new(),
                started_at: 0.0,
                animation_frame: None,
                play_button: element_by_id(document, "play")?,
                stop_button: element_by_id(document, "stop")?,
                bpm_input: element_by_id(document, "bpm")?,
                bpm_value: document
                    .get_element_by_id("bpmValue")
                    .ok_or_else(|| JsValue::from_str("#bpmValue element was not found"))?,
                status: document
                    .get_element_by_id("status")
                    .ok_or_else(|| JsValue::from_str("#status element was not found"))?,
            })
        }

        fn play(&mut self) -> Result<(), JsValue> {
            self.stop_current_source();

            let bpm = self.bpm_input.value().parse().unwrap_or(DEFAULT_BPM);
            self.synth.set_bpm(bpm);

            let context = self.audio_context()?;
            let _ = context.resume()?;

            let sample_rate = context.sample_rate();
            self.samples = self.synth.render_melody(sample_rate.round() as u32);

            let buffer = context.create_buffer(1, self.samples.len() as u32, sample_rate)?;
            buffer.copy_to_channel(&self.samples, 0)?;

            let source = context.create_buffer_source()?;
            source.set_buffer(Some(&buffer));
            source.connect_with_audio_node(&context.destination())?;

            self.started_at = context.current_time();
            source.start()?;
            self.source = Some(source);
            self.set_playing(true);
            self.set_status("Playing a Rust-synthesized melody.");

            Ok(())
        }

        fn draw_animation_frame(&mut self) -> Result<bool, JsValue> {
            if self.source.is_none() || self.samples.is_empty() {
                return Ok(false);
            }

            let context = self
                .audio_context
                .as_ref()
                .ok_or_else(|| JsValue::from_str("audio context is not available"))?;
            let duration = self.samples.len() as f64 / context.sample_rate() as f64;
            let progress = ((context.current_time() - self.started_at) / duration).clamp(0.0, 1.0);

            self.visualizer
                .draw_waveform(&self.samples, progress as f32)?;
            self.note_graph.draw(progress as f32)?;
            self.euler_graph.draw(progress as f32)?;

            if progress >= 1.0 {
                self.source = None;
                self.set_playing(false);
                self.note_graph.draw(1.0)?;
                self.euler_graph.draw(1.0)?;
                self.set_status("Finished. Adjust BPM and play again.");
                Ok(false)
            } else {
                Ok(true)
            }
        }

        fn stop_current_source(&mut self) {
            if let Some(source) = self.source.take() {
                let scheduled_source: &AudioScheduledSourceNode = source.unchecked_ref();
                let _ = scheduled_source.stop();
            }
            self.set_playing(false);
        }

        fn audio_context(&mut self) -> Result<AudioContext, JsValue> {
            if self.audio_context.is_none() {
                self.audio_context = Some(AudioContext::new()?);
            }

            Ok(self
                .audio_context
                .as_ref()
                .expect("audio context exists")
                .clone())
        }

        fn set_playing(&self, playing: bool) {
            self.play_button.set_disabled(playing);
            self.stop_button.set_disabled(!playing);
        }

        fn set_status(&self, message: &str) {
            self.status.set_text_content(Some(message));
        }
    }

    fn create_animation_loop(
        window: &Window,
        state: Rc<RefCell<AppState>>,
    ) -> Rc<RefCell<Option<Closure<dyn FnMut()>>>> {
        let animation: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
        let animation_for_frame = Rc::clone(&animation);
        let window_for_frame = window.clone();

        *animation.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            let keep_animating = {
                let mut state = state.borrow_mut();
                state.animation_frame = None;
                state.draw_animation_frame().unwrap_or(false)
            };

            if keep_animating {
                if let Some(callback) = animation_for_frame.borrow().as_ref() {
                    if let Ok(frame_id) =
                        window_for_frame.request_animation_frame(callback.as_ref().unchecked_ref())
                    {
                        state.borrow_mut().animation_frame = Some(frame_id);
                    }
                }
            }
        }) as Box<dyn FnMut()>));

        animation
    }

    fn bind_bpm_input(state: Rc<RefCell<AppState>>) -> Result<(), JsValue> {
        let input = state.borrow().bpm_input.clone();
        let input_for_listener = input.clone();
        let bpm_value = state.borrow().bpm_value.clone();
        let state_for_input = Rc::clone(&state);

        let on_input = Closure::wrap(Box::new(move || {
            let bpm = input_for_listener.value();
            bpm_value.set_text_content(Some(&bpm));

            if let Ok(bpm) = bpm.parse::<f32>() {
                state_for_input.borrow_mut().synth.set_bpm(bpm);
            }
        }) as Box<dyn FnMut()>);

        input.add_event_listener_with_callback("input", on_input.as_ref().unchecked_ref())?;
        on_input.forget();

        Ok(())
    }

    fn bind_play_button(
        window: &Window,
        state: Rc<RefCell<AppState>>,
        animation: Rc<RefCell<Option<Closure<dyn FnMut()>>>>,
    ) -> Result<(), JsValue> {
        let play_button = state.borrow().play_button.clone();
        let window_for_play = window.clone();
        let state_for_play = Rc::clone(&state);
        let animation_for_play = Rc::clone(&animation);

        let on_click = Closure::wrap(Box::new(move || {
            cancel_scheduled_animation(&window_for_play, &state_for_play);

            let play_result = state_for_play.borrow_mut().play();
            match play_result {
                Ok(()) => {
                    request_next_frame(&window_for_play, &state_for_play, &animation_for_play)
                }
                Err(_) => {
                    let mut state = state_for_play.borrow_mut();
                    state.stop_current_source();
                    state.set_status("Could not start audio. Try pressing Play again.");
                }
            }
        }) as Box<dyn FnMut()>);

        play_button.add_event_listener_with_callback("click", on_click.as_ref().unchecked_ref())?;
        on_click.forget();

        Ok(())
    }

    fn bind_stop_button(window: &Window, state: Rc<RefCell<AppState>>) -> Result<(), JsValue> {
        let stop_button = state.borrow().stop_button.clone();
        let window_for_stop = window.clone();
        let state_for_stop = Rc::clone(&state);

        let on_click = Closure::wrap(Box::new(move || {
            cancel_scheduled_animation(&window_for_stop, &state_for_stop);

            let mut state = state_for_stop.borrow_mut();
            state.stop_current_source();
            state.set_status("Stopped.");

            if state.samples.is_empty() {
                let _ = state.visualizer.draw_idle();
                let _ = state.note_graph.draw(0.0);
                let _ = state.euler_graph.draw(0.0);
            } else {
                let _ = state.visualizer.draw_waveform(&state.samples, 0.0);
                let _ = state.note_graph.draw(0.0);
                let _ = state.euler_graph.draw(0.0);
            }
        }) as Box<dyn FnMut()>);

        stop_button.add_event_listener_with_callback("click", on_click.as_ref().unchecked_ref())?;
        on_click.forget();

        Ok(())
    }

    fn request_next_frame(
        window: &Window,
        state: &Rc<RefCell<AppState>>,
        animation: &Rc<RefCell<Option<Closure<dyn FnMut()>>>>,
    ) {
        if let Some(callback) = animation.borrow().as_ref() {
            if let Ok(frame_id) = window.request_animation_frame(callback.as_ref().unchecked_ref())
            {
                state.borrow_mut().animation_frame = Some(frame_id);
            }
        }
    }

    fn cancel_scheduled_animation(window: &Window, state: &Rc<RefCell<AppState>>) {
        if let Some(frame_id) = state.borrow_mut().animation_frame.take() {
            let _ = window.cancel_animation_frame(frame_id);
        }
    }

    fn element_by_id<T>(document: &Document, id: &str) -> Result<T, JsValue>
    where
        T: JsCast,
    {
        document
            .get_element_by_id(id)
            .ok_or_else(|| JsValue::from_str(&format!("#{id} element was not found")))?
            .dyn_into::<T>()
            .map_err(|_| JsValue::from_str(&format!("#{id} element has the wrong type")))
    }
}

#[cfg(target_arch = "wasm32")]
mod visualizer {
    use wasm_bindgen::{JsCast, JsValue, prelude::*};
    use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, window};

    use super::MELODY;

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

    /// Circular linked graph inspired by Euler's Tonnetz and the circle of fifths.
    pub struct EulerGraphVisualizer {
        canvas: HtmlCanvasElement,
        ctx: CanvasRenderingContext2d,
    }

    impl EulerGraphVisualizer {
        pub fn new(canvas_id: &str) -> Result<EulerGraphVisualizer, JsValue> {
            let (canvas, ctx) = canvas_context(canvas_id, "Euler/Tonnetz graph canvas")?;
            Ok(EulerGraphVisualizer { canvas, ctx })
        }

        pub fn draw(&self, progress: f32) -> Result<(), JsValue> {
            let width = self.width();
            let height = self.height();
            let active_note = active_note_index(progress);
            let active_pitch_class = pitch_class_from_semitone(MELODY[active_note]);

            self.ctx.clear_rect(0.0, 0.0, width, height);
            self.draw_background(width, height)?;
            self.draw_relationship_links(width, height);
            self.draw_melody_trace(width, height, active_note);
            self.draw_nodes(width, height, active_pitch_class)?;
            self.draw_title(width, height, active_note)?;

            Ok(())
        }

        fn width(&self) -> f64 {
            self.canvas.width() as f64
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

        fn position_for_pitch_class(
            &self,
            pitch_class: i32,
            width: f64,
            height: f64,
        ) -> (f64, f64) {
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

        fn draw_melody_trace(&self, width: f64, height: f64, active_note: usize) {
            if active_note == 0 {
                return;
            }

            self.ctx.set_line_width(4.0);
            self.ctx.set_stroke_style_str("rgba(255, 223, 107, 0.58)");
            self.ctx.begin_path();

            for note_index in 0..=active_note.min(MELODY.len() - 1) {
                let pitch_class = pitch_class_from_semitone(MELODY[note_index]);
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
            active_pitch_class: i32,
        ) -> Result<(), JsValue> {
            for (pitch_class, name) in CIRCLE_OF_FIFTHS {
                let (x, y) = self.position_for_pitch_class(pitch_class, width, height);
                let is_active = pitch_class == active_pitch_class;
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

        fn draw_title(&self, width: f64, height: f64, active_note: usize) -> Result<(), JsValue> {
            let active = MELODY[active_note.min(MELODY.len() - 1)];
            let pitch_class = pitch_class_from_semitone(active);

            self.ctx.set_font("14px system-ui, sans-serif");
            self.ctx.set_text_align("left");
            self.ctx.set_fill_style_str("rgba(245, 247, 251, 0.72)");
            self.ctx
                .fill_text("circle of fifths with Tonnetz-like third links", 24.0, 28.0)?;

            self.ctx.set_fill_style_str("#ffdf6b");
            self.ctx.fill_text(
                &format!("active pitch class: {}", pitch_class_name(pitch_class)),
                24.0,
                height - 18.0,
            )?;

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

    /// Canvas renderer for the melody as pitch over time.
    pub struct NoteGraphVisualizer {
        canvas: HtmlCanvasElement,
        ctx: CanvasRenderingContext2d,
        min_note: i32,
        max_note: i32,
    }

    impl NoteGraphVisualizer {
        pub fn new(canvas_id: &str) -> Result<NoteGraphVisualizer, JsValue> {
            let (canvas, ctx) = canvas_context(canvas_id, "note graph canvas")?;
            let min_note = *MELODY.iter().min().unwrap_or(&0);
            let max_note = *MELODY.iter().max().unwrap_or(&0);

            Ok(NoteGraphVisualizer {
                canvas,
                ctx,
                min_note,
                max_note,
            })
        }

        pub fn draw(&self, progress: f32) -> Result<(), JsValue> {
            let width = self.width();
            let height = self.height();
            let progress = progress.clamp(0.0, 1.0);
            let played_x = width * progress as f64;
            let active_note = active_note_index(progress);

            self.ctx.clear_rect(0.0, 0.0, width, height);
            self.draw_background(width, height)?;
            self.draw_grid(width, height);
            self.draw_note_path(width, height);
            self.draw_note_nodes(width, height, active_note);
            self.draw_playhead(played_x, height);
            self.draw_labels(width, height, active_note)?;

            Ok(())
        }

        fn width(&self) -> f64 {
            self.canvas.width() as f64
        }

        fn height(&self) -> f64 {
            self.canvas.height() as f64
        }

        fn x_for_note(&self, note_index: usize, width: f64) -> f64 {
            if MELODY.len() <= 1 {
                width / 2.0
            } else {
                let padding = 52.0;
                let usable_width = (width - padding * 2.0).max(1.0);
                padding + usable_width * note_index as f64 / (MELODY.len() - 1) as f64
            }
        }

        fn y_for_note(&self, semitone: i32, height: f64) -> f64 {
            let padding = 34.0;
            let usable_height = (height - padding * 2.0).max(1.0);
            let note_range = (self.max_note - self.min_note).max(1) as f64;
            let normalized = (semitone - self.min_note) as f64 / note_range;

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

        fn draw_grid(&self, width: f64, height: f64) {
            self.ctx.set_line_width(1.0);
            self.ctx.set_stroke_style_str("rgba(255, 255, 255, 0.10)");

            for semitone in self.min_note..=self.max_note {
                let y = self.y_for_note(semitone, height);
                self.ctx.begin_path();
                self.ctx.move_to(42.0, y);
                self.ctx.line_to(width - 18.0, y);
                self.ctx.stroke();
            }

            self.ctx.set_stroke_style_str("rgba(119, 247, 197, 0.12)");
            for note_index in 0..MELODY.len() {
                let x = self.x_for_note(note_index, width);
                self.ctx.begin_path();
                self.ctx.move_to(x, 24.0);
                self.ctx.line_to(x, height - 24.0);
                self.ctx.stroke();
            }
        }

        fn draw_note_path(&self, width: f64, height: f64) {
            self.ctx.set_line_width(3.0);
            self.ctx.set_stroke_style_str("#77f7c5");
            self.ctx.begin_path();

            for (note_index, semitone) in MELODY.iter().enumerate() {
                let x = self.x_for_note(note_index, width);
                let y = self.y_for_note(*semitone, height);

                if note_index == 0 {
                    self.ctx.move_to(x, y);
                } else {
                    self.ctx.line_to(x, y);
                }
            }

            self.ctx.stroke();
        }

        fn draw_note_nodes(&self, width: f64, height: f64, active_note: usize) {
            for (note_index, semitone) in MELODY.iter().enumerate() {
                let x = self.x_for_note(note_index, width);
                let y = self.y_for_note(*semitone, height);
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

        fn draw_labels(&self, width: f64, height: f64, active_note: usize) -> Result<(), JsValue> {
            self.ctx.set_font("14px system-ui, sans-serif");
            self.ctx.set_fill_style_str("rgba(245, 247, 251, 0.72)");
            self.ctx.set_text_align("right");

            for semitone in self.min_note..=self.max_note {
                let y = self.y_for_note(semitone, height) + 4.0;
                self.ctx.fill_text(&note_name(semitone), 34.0, y)?;
            }

            let semitone = MELODY[active_note.min(MELODY.len() - 1)];
            let frequency = super::frequency_for_semitone(semitone);
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

    fn canvas_context(
        canvas_id: &str,
        label: &str,
    ) -> Result<(HtmlCanvasElement, CanvasRenderingContext2d), JsValue> {
        let document = window()
            .ok_or_else(|| JsValue::from_str("window is not available"))?
            .document()
            .ok_or_else(|| JsValue::from_str("document is not available"))?;

        let canvas = document
            .get_element_by_id(canvas_id)
            .ok_or_else(|| JsValue::from_str(&format!("{label} was not found")))?
            .dyn_into::<HtmlCanvasElement>()?;

        let ctx = canvas
            .get_context("2d")?
            .ok_or_else(|| JsValue::from_str("2D canvas context is not available"))?
            .dyn_into::<CanvasRenderingContext2d>()?;

        Ok((canvas, ctx))
    }

    fn active_note_index(progress: f32) -> usize {
        let note_count = MELODY.len();
        if note_count == 0 {
            0
        } else {
            ((progress.clamp(0.0, 0.999_999) * note_count as f32).floor() as usize)
                .min(note_count - 1)
        }
    }

    fn pitch_class_from_semitone(semitone_from_a4: i32) -> i32 {
        (69 + semitone_from_a4).rem_euclid(12)
    }

    fn pitch_class_name(pitch_class: i32) -> &'static str {
        const NAMES: [&str; 12] = [
            "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
        ];

        NAMES[pitch_class.rem_euclid(12) as usize]
    }

    fn note_name(semitone_from_a4: i32) -> String {
        let midi_note = 69 + semitone_from_a4;
        let name = pitch_class_name(midi_note.rem_euclid(12));
        let octave = midi_note.div_euclid(12) - 1;

        format!("{name}{octave}")
    }

    /// Canvas renderer for the generated audio waveform.
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
                "Rust UI is ready — press Play melody",
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
