//! Rust + WebAssembly music playground.
//!
//! `zegois_music` synthesizes short melodies from sine waves, plays them in
//! the browser through Web Audio, and draws music-oriented canvas
//! visualizations. Notes are represented throughout the crate as semitone
//! offsets from A4 (`0 == 440 Hz`), which keeps the synth, graph, piano, and
//! guitar-neck editors speaking the same musical language.
//!
//! The native API exposes the small synth core for testing and reuse. The
//! browser UI and visualizers are compiled only for `wasm32` targets.

mod synth;

pub use synth::{Synth, frequency_for_semitone, render_melody};

#[cfg(target_arch = "wasm32")]
pub(crate) use synth::{DEFAULT_BPM, DEFAULT_MELODY, render_notes};

#[cfg(target_arch = "wasm32")]
mod app;
#[cfg(target_arch = "wasm32")]
mod visualizer;

#[cfg(target_arch = "wasm32")]
pub use app::start_app;
#[cfg(target_arch = "wasm32")]
pub use visualizer::WaveformVisualizer;
