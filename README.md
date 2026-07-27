# Zegóis Music

A minimal Rust + WebAssembly music MVP: Rust renders a sine-wave melody using equal temperament, the browser plays it with the Web Audio API, and Rust/WASM draws the generated waveform on a canvas.

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

- `src/lib.rs` exposes `Synth` and `WaveformVisualizer` through `wasm-bindgen`.
- Frequencies are calculated with `440 * 2^(n / 12)`, where `n` is semitones from A4.
- The synth renders mono `f32` PCM samples.
- `web/main.js` copies those samples into a Web Audio `AudioBuffer`.
- Rust/WASM draws the same samples on the `web/index.html` canvas.
