//! Rust-owned HTML template for the browser app.
//!
//! Keeping the markup in Rust makes `start_app` the single entry point for UI
//! construction and event binding. The static page only needs an `#app` mount
//! node and the WASM loader.

/// Markup injected into the page root by `start_app`.
pub(super) const APP_HTML: &str = r#"
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
        <input id="replaceMode" type="radio" name="editMode" value="replace" checked />
        Replace selected step
      </label>
      <label>
        <input id="insertMode" type="radio" name="editMode" value="insert" />
        Insert after selected step
      </label>
      <label>
        <input id="appendMode" type="radio" name="editMode" value="append" />
        Append to melody
      </label>
    </fieldset>
    <p id="melodyStatus" class="melody-status">32 notes · click instruments or graph nodes to edit.</p>
    <p id="selectedNoteStatus" class="melody-status">Editing step 1.</p>

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
      <div>
        <h2>Manual note editor · piano</h2>
        <canvas id="pianoKeyboard" width="960" height="180" aria-label="Clickable piano keyboard note editor"></canvas>
      </div>
      <div>
        <h2>Manual note editor · guitar neck</h2>
        <canvas id="guitarNeck" width="960" height="260" aria-label="Clickable guitar neck note editor"></canvas>
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
      <li>Click pitch-class nodes, piano keys, or guitar frets to audition notes and edit the melody.</li>
      <li>Use edit mode to replace the selected instant, insert after it, or append to the melody.</li>
    </ul>
  </section>
</main>
"#;
