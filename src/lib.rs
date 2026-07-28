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
