# Zegóis Music

A minimal Rust + WebAssembly music MVP: Rust builds the UI, renders a sine-wave melody using equal temperament, plays it through the browser Web Audio API, draws the generated waveform, graphs the melody notes as pitch over time, and visualizes pitch-class relationships in an Euler/Tonnetz-inspired circular graph.

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
- `web/main.js` is now only a tiny WASM loader.
