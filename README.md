# Zegóis Music

A minimal Rust + WebAssembly music MVP: Rust builds the UI, renders a sine-wave melody using equal temperament, plays it through the browser Web Audio API, and draws the generated waveform on a canvas.

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
- Rust/WASM draws the same samples on the generated canvas.
- `web/main.js` is now only a tiny WASM loader.
