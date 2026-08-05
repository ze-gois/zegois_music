//! Rust-owned HTML template for the browser app.
//!
//! Keeping the markup in Rust makes `start_app` the single entry point for UI
//! construction and event binding. The static page only needs an `#app` mount
//! node and the WASM loader.

pub mod button;
pub mod custom;
pub mod events;
pub mod input;
pub mod state;

pub use state::AppState;

pub use crate::shared::graph::WaveformVisualizer;

use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{JsValue, prelude::*};
use web_sys::Window;
use web_sys::window;

pub use humans::audible::music;
pub use music::synth::{DEFAULT_BPM, DEFAULT_MELODY, render_notes};
pub use music::synth::{Synth, frequency_for_semitone, render_melody};

/// Markup injected into the page root by `start_app`.
pub const APP_HTML: &str = r#"
<main>
  <section class="hero">
    <p class="eyebrow">𝄠𝄠𝄠 Rust + WebAssembly + Web Audio 𝄠𝄠𝄠</p>
    <h1>Music</h1>
    <p>
      A tiny synthesized melody rendered from sine waves in Rust, played in
      the browser, visualized on canvas, and now controlled by a Rust-built UI.
    </p>
  </section>

  <section class="panel">
    <div class="visual-stack">
      <div>
        <h2>Euler/Tonnetz graph</h2>
        <canvas id="eulerGraph" width="960" height="360" aria-label="Circular linked graph of pitch-class relationships"></canvas>
      </div>
      <div>
        <h2>Manual note editor · piano</h2>
        <canvas id="pianoKeyboard" width="960" height="180" aria-label="Clickable piano keyboard note editor"></canvas>
      </div>
      <div>
        <h2>Manual note editor · guitar neck</h2>
        <canvas id="guitarNeck" width="960" height="260" aria-label="Clickable guitar neck note editor"></canvas>
      </div>
      <div>
        <h2>Note graph</h2>
        <canvas id="noteGraph" width="960" height="260" aria-label="Pitch over time note graph"></canvas>
      </div>
      <div>
        <h2>Waveform</h2>
        <canvas id="visualizer" width="960" height="320" aria-label="Waveform visualizer"></canvas>
      </div>
    </div>
    <p id="status" class="status">Ready. Press Play melody to synthesize sound from Rust.</p>
    <div class="controls">
      <button id="play">Play melody</button>
      <button id="stop" disabled>Stop!</button>
      <label>
        BPM
        <input id="bpm" type="range" min="60" max="180" value="108" />
        <span id="bpmValue">108</span>
      </label>
      <button id="resetMelody">Reset melody</button>
      <button id="walkMelody">Generate graph walk</button>
      <button id="clearMelody">Clear melody</button>
      <label>
        Note step
        <input id="noteStep" type="range" min="1" max="32" value="1" />
        <span id="noteStepValue">1</span>
      </label>
    </div>
    <fieldset class="edit-mode">
      <legend>Edit melody mode</legend>
      <label>
        <input id="replaceMode" type="radio" name="editMode" value="replace" />
        Replace selected step
      </label>
      <label>
        <input id="insertMode" type="radio" name="editMode" value="insert" checked/>
        Insert after selected step
      </label>
      <label>
        <input id="appendMode" type="radio" name="editMode" value="append" />
        Append to melody
      </label>
    </fieldset>
    <p id="melodyStatus" class="melody-status">32 notes · click instruments or graph nodes to edit.</p>
    <p id="selectedNoteStatus" class="melody-status">Editing step 1.</p>

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
      <li>Click pitch-class nodes, piano keys, or guitar frets to audition notes and edit the melody.</li>
      <li>Use edit mode to replace the selected instant, insert after it, or append to the melody.</li>
    </ul>
  </section>
</main>
"#;

/// Mount the Rust-owned music UI into `#app` and bind browser events.
///
/// This is the main function called by `web/main.js` after the WASM package is
/// loaded. It returns a JavaScript error value when required DOM APIs or
/// elements are unavailable.
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
    state.borrow().redraw_all_idle()?;

    let animation = events::create_animation_loop(&window, Rc::clone(&state));
    state.borrow().update_note_step_ui();
    state.borrow().update_melody_status();

    button::play::bind::click(&window, Rc::clone(&state), Rc::clone(&animation))?;
    button::stop::bind::click(&window, Rc::clone(&state))?;
    button::reset::bind::click(Rc::clone(&state))?;
    button::walk::bind::click(Rc::clone(&state))?;
    button::clear::bind::click(Rc::clone(&state))?;

    input::bpm::bind::click(Rc::clone(&state))?;
    input::note_step::bind::click(Rc::clone(&state))?;
    input::edit_mode::bind::click(&document, Rc::clone(&state))?;

    custom::euler::bind::click(&window, Rc::clone(&state))?;
    custom::piano::bind::click(&window, Rc::clone(&state))?;
    custom::guitar::bind::click(&window, Rc::clone(&state))?;

    Ok(())
}

fn request_next_frame(
    window: &Window,
    state: &Rc<RefCell<AppState>>,
    animation: &Rc<RefCell<Option<Closure<dyn FnMut()>>>>,
) {
    if let Some(callback) = animation.borrow().as_ref() {
        if let Ok(frame_id) = window.request_animation_frame(callback.as_ref().unchecked_ref()) {
            state.borrow_mut().animation_frame = Some(frame_id);
        }
    }
}

fn cancel_scheduled_animation(window: &Window, state: &Rc<RefCell<AppState>>) {
    if let Some(frame_id) = state.borrow_mut().animation_frame.take() {
        let _ = window.cancel_animation_frame(frame_id);
    }
}
