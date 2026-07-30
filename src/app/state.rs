//! Mutable browser app state.
//!
//! `AppState` is the center of the interactive composer. It owns the synth,
//! audio nodes, current melody, selected edit step, edit mode, controls, and all
//! visualizers. Event handlers borrow it through `Rc<RefCell<_>>`, keeping DOM
//! closures small while centralizing state transitions here.

use wasm_bindgen::{JsCast, JsValue};
use web_sys::{
    AudioBufferSourceNode, AudioContext, AudioScheduledSourceNode, Document, Element,
    HtmlButtonElement, HtmlInputElement,
};

use crate::{
    DEFAULT_BPM, DEFAULT_MELODY, Synth, frequency_for_semitone, render_notes,
    visualizer::{
        EulerGraphVisualizer, GuitarNeckVisualizer, NoteGraphVisualizer, PianoKeyboardVisualizer,
        WaveformVisualizer, note_name,
    },
};

use super::{dom::element_by_id, music::graph_walk_melody, music::semitone_for_pitch_class_near};

/// How a clicked note changes the melody.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum EditMode {
    Replace,
    Insert,
    Append,
}

impl EditMode {
    fn label(self) -> &'static str {
        match self {
            EditMode::Replace => "replace selected step",
            EditMode::Insert => "insert after selected step",
            EditMode::Append => "append to melody",
        }
    }
}

/// Runtime state for playback, composition, and visual feedback.
///
/// Notes are stored as semitone offsets from A4. The selected note index points
/// to the melody instant that replace/insert operations act on.
pub(super) struct AppState {
    pub(super) synth: Synth,
    pub(super) visualizer: WaveformVisualizer,
    pub(super) note_graph: NoteGraphVisualizer,
    pub(super) euler_graph: EulerGraphVisualizer,
    pub(super) piano: PianoKeyboardVisualizer,
    pub(super) guitar: GuitarNeckVisualizer,
    audio_context: Option<AudioContext>,
    source: Option<AudioBufferSourceNode>,
    preview_source: Option<AudioBufferSourceNode>,
    pub(super) samples: Vec<f32>,
    pub(super) melody: Vec<i32>,
    selected_note_index: usize,
    edit_mode: EditMode,
    started_at: f64,
    pub(super) animation_frame: Option<i32>,
    pub(super) play_button: HtmlButtonElement,
    pub(super) stop_button: HtmlButtonElement,
    pub(super) bpm_input: HtmlInputElement,
    pub(super) bpm_value: Element,
    melody_status: Element,
    selected_note_status: Element,
    status: Element,
    pub(super) note_step_input: HtmlInputElement,
    pub(super) note_step_value: Element,
    pub(super) reset_button: HtmlButtonElement,
    pub(super) walk_button: HtmlButtonElement,
    pub(super) clear_button: HtmlButtonElement,
}

impl AppState {
    /// Find required DOM elements and construct all visualizers.
    pub(super) fn new(document: &Document) -> Result<Self, JsValue> {
        Ok(Self {
            synth: Synth::new(),
            visualizer: WaveformVisualizer::new("visualizer")?,
            note_graph: NoteGraphVisualizer::new("noteGraph")?,
            euler_graph: EulerGraphVisualizer::new("eulerGraph")?,
            piano: PianoKeyboardVisualizer::new("pianoKeyboard")?,
            guitar: GuitarNeckVisualizer::new("guitarNeck")?,
            audio_context: None,
            source: None,
            preview_source: None,
            samples: Vec::new(),
            melody: DEFAULT_MELODY.to_vec(),
            selected_note_index: 0,
            edit_mode: EditMode::Replace,
            started_at: 0.0,
            animation_frame: None,
            play_button: element_by_id(document, "play")?,
            stop_button: element_by_id(document, "stop")?,
            bpm_input: element_by_id(document, "bpm")?,
            bpm_value: document
                .get_element_by_id("bpmValue")
                .ok_or_else(|| JsValue::from_str("#bpmValue element was not found"))?,
            melody_status: document
                .get_element_by_id("melodyStatus")
                .ok_or_else(|| JsValue::from_str("#melodyStatus element was not found"))?,
            selected_note_status: document
                .get_element_by_id("selectedNoteStatus")
                .ok_or_else(|| JsValue::from_str("#selectedNoteStatus element was not found"))?,
            status: document
                .get_element_by_id("status")
                .ok_or_else(|| JsValue::from_str("#status element was not found"))?,
            reset_button: element_by_id(document, "resetMelody")?,
            walk_button: element_by_id(document, "walkMelody")?,
            clear_button: element_by_id(document, "clearMelody")?,
            note_step_input: element_by_id(document, "noteStep")?,
            note_step_value: document
                .get_element_by_id("noteStepValue")
                .ok_or_else(|| JsValue::from_str("#noteStepValue element was not found"))?,
        })
    }

    /// Render the current melody and start Web Audio playback.
    pub(super) fn play(&mut self) -> Result<(), JsValue> {
        self.stop_current_source();
        self.stop_preview_source();

        let bpm = self.bpm_input.value().parse().unwrap_or(DEFAULT_BPM);
        self.synth.set_bpm(bpm);

        let context = self.audio_context()?;
        let _ = context.resume()?;

        let sample_rate = context.sample_rate();
        if self.melody.is_empty() {
            self.set_status("Add notes first by clicking the graph, piano, or guitar neck.");
            return Ok(());
        }

        self.samples = render_notes(sample_rate.round() as u32, self.synth.bpm(), &self.melody);

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

    /// Redraw playback-driven visualizers for one animation frame.
    ///
    /// Returns `true` while playback should continue requesting frames.
    pub(super) fn draw_animation_frame(&mut self) -> Result<bool, JsValue> {
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
        self.note_graph.draw(&self.melody, progress as f32)?;
        self.euler_graph.draw(&self.melody, progress as f32)?;
        self.piano.draw(self.selected_note())?;
        self.guitar.draw(self.selected_note())?;

        if progress >= 1.0 {
            self.source = None;
            self.set_playing(false);
            self.note_graph.draw(&self.melody, 1.0)?;
            self.euler_graph.draw(&self.melody, 1.0)?;
            self.piano.draw(self.selected_note())?;
            self.guitar.draw(self.selected_note())?;
            self.set_status("Finished. Adjust BPM and play again.");
            Ok(false)
        } else {
            Ok(true)
        }
    }

    pub(super) fn stop_current_source(&mut self) {
        if let Some(source) = self.source.take() {
            stop_source(&source);
        }
        self.set_playing(false);
    }

    pub(super) fn stop_preview_source(&mut self) {
        if let Some(source) = self.preview_source.take() {
            stop_source(&source);
        }
    }

    fn audition_note(&mut self, semitone: i32) -> Result<(), JsValue> {
        self.stop_preview_source();

        let context = self.audio_context()?;
        let _ = context.resume()?;
        let sample_rate = context.sample_rate();
        let samples = render_notes(sample_rate.round() as u32, 180.0, &[semitone]);

        let buffer = context.create_buffer(1, samples.len() as u32, sample_rate)?;
        buffer.copy_to_channel(&samples, 0)?;

        let source = context.create_buffer_source()?;
        source.set_buffer(Some(&buffer));
        source.connect_with_audio_node(&context.destination())?;
        source.start()?;
        self.preview_source = Some(source);

        Ok(())
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

    fn selected_note(&self) -> Option<i32> {
        self.melody.get(self.selected_note_index).copied()
    }

    fn selected_progress(&self) -> f32 {
        if self.melody.len() <= 1 {
            0.0
        } else {
            self.selected_note_index.min(self.melody.len() - 1) as f32
                / (self.melody.len() - 1) as f32
        }
    }

    pub(super) fn select_note_index(&mut self, note_index: usize) -> Result<(), JsValue> {
        self.selected_note_index = note_index.min(self.melody.len().saturating_sub(1));
        self.update_note_step_ui();
        self.redraw_melody_graphs(self.selected_progress())?;
        self.update_melody_status();
        Ok(())
    }

    pub(super) fn set_edit_mode(&mut self, mode: EditMode) {
        self.edit_mode = mode;
        self.update_melody_status();
        self.set_status(&format!("Edit mode: {}.", mode.label()));
    }

    /// Audition a clicked instrument note and apply the active edit mode.
    pub(super) fn apply_manual_note(&mut self, semitone: i32) -> Result<(), JsValue> {
        self.stop_current_source();
        self.samples.clear();
        self.audition_note(semitone)?;

        match self.edit_mode {
            EditMode::Replace => self.replace_selected_note(semitone),
            EditMode::Insert => self.insert_after_selected_note(semitone),
            EditMode::Append => self.append_note(semitone),
        }

        self.update_note_step_ui();
        self.redraw_melody_graphs(self.selected_progress())?;
        self.update_melody_status();
        self.set_status(&format!(
            "Played {} and updated melody: {}.",
            note_name(semitone),
            self.edit_mode.label()
        ));
        Ok(())
    }

    fn replace_selected_note(&mut self, semitone: i32) {
        if self.melody.is_empty() {
            self.melody.push(semitone);
            self.selected_note_index = 0;
        } else {
            let index = self.selected_note_index.min(self.melody.len() - 1);
            self.melody[index] = semitone;
            self.selected_note_index = index;
        }
    }

    fn insert_after_selected_note(&mut self, semitone: i32) {
        if self.melody.is_empty() {
            self.melody.push(semitone);
            self.selected_note_index = 0;
        } else {
            let index = self.selected_note_index.min(self.melody.len() - 1) + 1;
            self.melody.insert(index, semitone);
            self.selected_note_index = index;
        }
    }

    fn append_note(&mut self, semitone: i32) {
        self.melody.push(semitone);
        self.selected_note_index = self.melody.len() - 1;
    }

    /// Convert a clicked graph pitch class to a nearby concrete note and edit.
    pub(super) fn apply_pitch_class(&mut self, pitch_class: i32) -> Result<(), JsValue> {
        let reference = match self.edit_mode {
            EditMode::Replace | EditMode::Insert => self.selected_note().unwrap_or(0),
            EditMode::Append => self.melody.last().copied().unwrap_or(0),
        };
        let semitone = semitone_for_pitch_class_near(pitch_class, reference);
        self.apply_manual_note(semitone)
    }

    pub(super) fn reset_melody(&mut self) -> Result<(), JsValue> {
        self.stop_current_source();
        self.stop_preview_source();
        self.samples.clear();
        self.melody = DEFAULT_MELODY.to_vec();
        self.selected_note_index = 0;
        self.update_note_step_ui();
        self.redraw_all_idle()?;
        self.update_melody_status();
        self.set_status("Reset to the original melody.");
        Ok(())
    }

    pub(super) fn clear_melody(&mut self) -> Result<(), JsValue> {
        self.stop_current_source();
        self.stop_preview_source();
        self.samples.clear();
        self.melody.clear();
        self.selected_note_index = 0;
        self.update_note_step_ui();
        self.visualizer.draw_idle()?;
        self.redraw_melody_graphs(0.0)?;
        self.update_melody_status();
        self.set_status(
            "Melody cleared. Click a graph node, piano key, or guitar fret to compose.",
        );
        Ok(())
    }

    pub(super) fn generate_graph_walk(&mut self) -> Result<(), JsValue> {
        self.stop_current_source();
        self.stop_preview_source();
        self.samples.clear();
        self.melody = graph_walk_melody();
        self.selected_note_index = 0;
        self.update_note_step_ui();
        self.redraw_all_idle()?;
        self.update_melody_status();
        self.set_status("Generated a melody by walking fifth/third relationships.");
        Ok(())
    }

    pub(super) fn redraw_all_idle(&self) -> Result<(), JsValue> {
        self.visualizer.draw_idle()?;
        self.redraw_melody_graphs(0.0)
    }

    fn redraw_melody_graphs(&self, progress: f32) -> Result<(), JsValue> {
        self.note_graph.draw(&self.melody, progress)?;
        self.euler_graph.draw(&self.melody, progress)?;
        self.piano.draw(self.selected_note())?;
        self.guitar.draw(self.selected_note())?;
        Ok(())
    }

    pub(super) fn update_melody_status(&self) {
        let message = if self.melody.is_empty() {
            "0 notes · click a graph node, piano key, or guitar fret to start composing."
                .to_string()
        } else {
            let preview = self
                .melody
                .iter()
                .rev()
                .take(6)
                .map(|note| note_name(*note))
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
                .join(" → ");
            format!(
                "{} notes · recent: {} · mode: {}.",
                self.melody.len(),
                preview,
                self.edit_mode.label()
            )
        };
        self.melody_status.set_text_content(Some(&message));
        self.update_selected_note_status();
    }

    pub(super) fn update_note_step_ui(&self) {
        let max = self.melody.len().max(1).to_string();
        let selected = self
            .selected_note_index
            .min(self.melody.len().saturating_sub(1));
        let value = (selected + 1).to_string();
        self.note_step_input.set_max(&max);
        self.note_step_input.set_value(&value);
        self.note_step_value.set_text_content(Some(&value));
        self.update_selected_note_status();
    }

    fn update_selected_note_status(&self) {
        let message = if let Some(note) = self.selected_note() {
            format!(
                "Editing step {} · {} · {:.1} Hz · mode: {}",
                self.selected_note_index + 1,
                note_name(note),
                frequency_for_semitone(note),
                self.edit_mode.label()
            )
        } else {
            format!(
                "No note selected · mode: {} · click a piano key, guitar fret, or graph node.",
                self.edit_mode.label()
            )
        };
        self.selected_note_status.set_text_content(Some(&message));
    }

    pub(super) fn set_status(&self, message: &str) {
        self.status.set_text_content(Some(message));
    }
}

fn stop_source(source: &AudioBufferSourceNode) {
    let scheduled_source: &AudioScheduledSourceNode = source.unchecked_ref();
    let _ = scheduled_source.stop();
}
