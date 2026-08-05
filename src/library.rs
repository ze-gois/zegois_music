//! Rust + WebAssembly music playground.
//! Browser application entry point.
//!
//! This module owns startup: it injects the Rust-generated HTML, constructs
//! [`AppState`], draws the initial visualizers, and binds all DOM/canvas events.
//! The submodules keep the UI template, event closures, state transitions, and
//! music helpers separate so the WASM app remains readable as it grows.
//!
//! `zegois_music` synthesizes short melodies from sine waves, plays them in
//! the browser through Web Audio, and draws music-oriented canvas
//! visualizations. Notes are represented throughout the crate as semitone
//! offsets from A4 (`0 == 440 Hz`), which keeps the synth, graph, piano, and
//! guitar-neck editors speaking the same musical language.
//!
//! The native API exposes the small synth core for testing and reuse. The
//! browser UI and visualizers are compiled only for `wasm32` targets.

#![allow(special_module_name)]
pub mod main;
pub mod shared;

pub use main::start_app;

// #[cfg(target_arch = "wasm32")]

// #[cfg(target_arch = "wasm32")]
// #[cfg(target_arch = "wasm32")]

// #[cfg(target_arch = "wasm32")]
// #[cfg(target_arch = "wasm32")]
