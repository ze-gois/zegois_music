# Zegóis Music

A minimal Rust + WebAssembly music MVP: Rust builds the UI, lets you compose by clicking an Euler/Tonnetz-inspired pitch graph, edit individual melody instants with piano and guitar-neck canvases, renders sine-wave melodies using equal temperament, plays them through the browser Web Audio API, draws the generated waveform, and graphs the melody notes as pitch over time.

## Requirements

Install the WebAssembly Rust target and `wasm-pack` once:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
```

## Build and run

Build the WASM package into `web/pkg`:

```sh
wasm-pack build --target web --out-dir web/pkg
```

Serve the project directory with any static file server. For example:

```sh
python3 -m http.server 8080
```

Then open:

```text
http://localhost:8080/web/
```

> Browsers require a user gesture before audio can start, so press **Play melody** on the page.

## How it works

- `src/lib.rs` exposes `start_app`, `Synth`, and `WaveformVisualizer` through `wasm-bindgen`.
- `start_app` renders the controls into `#app`, binds events, and manages playback state.
- Frequencies are calculated with `440 * 2^(n / 12)`, where `n` is semitones from A4.
- The synth renders mono `f32` PCM samples.
- Rust/WASM copies those samples into a Web Audio `AudioBuffer` and starts playback.
- Rust/WASM draws the same samples on the generated waveform canvas.
- Rust/WASM also draws a note graph showing pitch over time and the active note.
- Rust/WASM draws an Euler/Tonnetz-inspired linked graph: pitch classes are placed around the circle of fifths, with fifth/third relationships drawn as edges.
- Click pitch-class nodes to append notes to the melody.
- Select a **Note step**, then choose an **Edit melody mode**:
  - **Replace selected step** edits that exact instant.
  - **Insert after selected step** inserts a new note after it.
  - **Append to melody** adds a note to the end.
- Click the piano keyboard, guitar neck, or Euler/Tonnetz graph to audition the note and apply the selected edit mode.
- Use **Reset melody**, **Generate graph walk**, or **Clear melody** to change the composition.
- `web/main.js` is now only a tiny WASM loader.

## Documentation

Generate local Rust API documentation:

```sh
cargo doc --no-deps --document-private-items
```

For the browser/WASM-only API and modules, generate docs for the WebAssembly target:

```sh
cargo doc --target wasm32-unknown-unknown --no-deps --document-private-items
```

Open the generated docs at:

- Native docs: `target/doc/zegois_music/index.html`
- WASM docs: `target/wasm32-unknown-unknown/doc/zegois_music/index.html`

## Code structure

- `src/lib.rs` exposes the public WASM/native entry points and wires modules together.
- `src/synth.rs` contains sample generation, BPM handling, equal-temperament frequency math, and synth tests.
- `src/app/` owns the Rust-generated DOM, app state, event bindings, playback orchestration, edit modes, and graph-walk melody helpers.
- `src/visualizer/` contains canvas drawing and hit-testing for the waveform, note graph, Euler/Tonnetz graph, piano keyboard, and guitar neck.
