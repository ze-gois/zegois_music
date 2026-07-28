use std::{cell::RefCell, rc::Rc};

use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use web_sys::{Document, HtmlCanvasElement, HtmlInputElement, MouseEvent, Window};

use super::{
    dom::element_by_id,
    state::{AppState, EditMode},
};

pub(super) fn create_animation_loop(
    window: &Window,
    state: Rc<RefCell<AppState>>,
) -> Rc<RefCell<Option<Closure<dyn FnMut()>>>> {
    let animation: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let animation_for_frame = Rc::clone(&animation);
    let window_for_frame = window.clone();

    *animation.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let keep_animating = {
            let mut state = state.borrow_mut();
            state.animation_frame = None;
            state.draw_animation_frame().unwrap_or(false)
        };

        if keep_animating {
            if let Some(callback) = animation_for_frame.borrow().as_ref() {
                if let Ok(frame_id) =
                    window_for_frame.request_animation_frame(callback.as_ref().unchecked_ref())
                {
                    state.borrow_mut().animation_frame = Some(frame_id);
                }
            }
        }
    }) as Box<dyn FnMut()>));

    animation
}

pub(super) fn bind_bpm_input(state: Rc<RefCell<AppState>>) -> Result<(), JsValue> {
    let input = state.borrow().bpm_input.clone();
    let input_for_listener = input.clone();
    let bpm_value = state.borrow().bpm_value.clone();
    let state_for_input = Rc::clone(&state);

    let on_input = Closure::wrap(Box::new(move || {
        let bpm = input_for_listener.value();
        bpm_value.set_text_content(Some(&bpm));

        if let Ok(bpm) = bpm.parse::<f32>() {
            state_for_input.borrow_mut().synth.set_bpm(bpm);
        }
    }) as Box<dyn FnMut()>);

    input.add_event_listener_with_callback("input", on_input.as_ref().unchecked_ref())?;
    on_input.forget();

    Ok(())
}

pub(super) fn bind_play_button(
    window: &Window,
    state: Rc<RefCell<AppState>>,
    animation: Rc<RefCell<Option<Closure<dyn FnMut()>>>>,
) -> Result<(), JsValue> {
    let play_button = state.borrow().play_button.clone();
    let window_for_play = window.clone();
    let state_for_play = Rc::clone(&state);
    let animation_for_play = Rc::clone(&animation);

    let on_click = Closure::wrap(Box::new(move || {
        cancel_scheduled_animation(&window_for_play, &state_for_play);

        let play_result = state_for_play.borrow_mut().play();
        match play_result {
            Ok(()) => request_next_frame(&window_for_play, &state_for_play, &animation_for_play),
            Err(_) => {
                let mut state = state_for_play.borrow_mut();
                state.stop_current_source();
                state.set_status("Could not start audio. Try pressing Play again.");
            }
        }
    }) as Box<dyn FnMut()>);

    play_button.add_event_listener_with_callback("click", on_click.as_ref().unchecked_ref())?;
    on_click.forget();

    Ok(())
}

pub(super) fn bind_stop_button(
    window: &Window,
    state: Rc<RefCell<AppState>>,
) -> Result<(), JsValue> {
    let stop_button = state.borrow().stop_button.clone();
    let window_for_stop = window.clone();
    let state_for_stop = Rc::clone(&state);

    let on_click = Closure::wrap(Box::new(move || {
        cancel_scheduled_animation(&window_for_stop, &state_for_stop);

        let mut state = state_for_stop.borrow_mut();
        state.stop_current_source();
        state.stop_preview_source();
        state.set_status("Stopped.");

        if state.samples.is_empty() {
            let _ = state.visualizer.draw_idle();
            let _ = state.note_graph.draw(&state.melody, 0.0);
            let _ = state.euler_graph.draw(&state.melody, 0.0);
        } else {
            let _ = state.visualizer.draw_waveform(&state.samples, 0.0);
            let _ = state.note_graph.draw(&state.melody, 0.0);
            let _ = state.euler_graph.draw(&state.melody, 0.0);
        }
    }) as Box<dyn FnMut()>);

    stop_button.add_event_listener_with_callback("click", on_click.as_ref().unchecked_ref())?;
    on_click.forget();

    Ok(())
}

pub(super) fn bind_reset_button(state: Rc<RefCell<AppState>>) -> Result<(), JsValue> {
    let button = state.borrow().reset_button.clone();
    let state_for_click = Rc::clone(&state);

    let on_click = Closure::wrap(Box::new(move || {
        let _ = state_for_click.borrow_mut().reset_melody();
    }) as Box<dyn FnMut()>);

    button.add_event_listener_with_callback("click", on_click.as_ref().unchecked_ref())?;
    on_click.forget();

    Ok(())
}

pub(super) fn bind_walk_button(state: Rc<RefCell<AppState>>) -> Result<(), JsValue> {
    let button = state.borrow().walk_button.clone();
    let state_for_click = Rc::clone(&state);

    let on_click = Closure::wrap(Box::new(move || {
        let _ = state_for_click.borrow_mut().generate_graph_walk();
    }) as Box<dyn FnMut()>);

    button.add_event_listener_with_callback("click", on_click.as_ref().unchecked_ref())?;
    on_click.forget();

    Ok(())
}

pub(super) fn bind_clear_button(state: Rc<RefCell<AppState>>) -> Result<(), JsValue> {
    let button = state.borrow().clear_button.clone();
    let state_for_click = Rc::clone(&state);

    let on_click = Closure::wrap(Box::new(move || {
        let _ = state_for_click.borrow_mut().clear_melody();
    }) as Box<dyn FnMut()>);

    button.add_event_listener_with_callback("click", on_click.as_ref().unchecked_ref())?;
    on_click.forget();

    Ok(())
}

pub(super) fn bind_note_step_input(state: Rc<RefCell<AppState>>) -> Result<(), JsValue> {
    let input = state.borrow().note_step_input.clone();
    let input_for_listener = input.clone();
    let state_for_input = Rc::clone(&state);

    let on_input = Closure::wrap(Box::new(move || {
        let note_index = input_for_listener
            .value()
            .parse::<usize>()
            .unwrap_or(1)
            .saturating_sub(1);
        let _ = state_for_input.borrow_mut().select_note_index(note_index);
    }) as Box<dyn FnMut()>);

    input.add_event_listener_with_callback("input", on_input.as_ref().unchecked_ref())?;
    on_input.forget();

    Ok(())
}

pub(super) fn bind_edit_mode_inputs(
    document: &Document,
    state: Rc<RefCell<AppState>>,
) -> Result<(), JsValue> {
    bind_edit_mode_input(
        element_by_id(document, "replaceMode")?,
        Rc::clone(&state),
        EditMode::Replace,
    )?;
    bind_edit_mode_input(
        element_by_id(document, "insertMode")?,
        Rc::clone(&state),
        EditMode::Insert,
    )?;
    bind_edit_mode_input(
        element_by_id(document, "appendMode")?,
        state,
        EditMode::Append,
    )?;

    Ok(())
}

fn bind_edit_mode_input(
    input: HtmlInputElement,
    state: Rc<RefCell<AppState>>,
    mode: EditMode,
) -> Result<(), JsValue> {
    let input_for_listener = input.clone();
    let state_for_input = Rc::clone(&state);

    let on_change = Closure::wrap(Box::new(move || {
        if input_for_listener.checked() {
            state_for_input.borrow_mut().set_edit_mode(mode);
        }
    }) as Box<dyn FnMut()>);

    input.add_event_listener_with_callback("change", on_change.as_ref().unchecked_ref())?;
    on_change.forget();

    Ok(())
}

pub(super) fn bind_euler_click(
    window: &Window,
    state: Rc<RefCell<AppState>>,
) -> Result<(), JsValue> {
    let canvas = state.borrow().euler_graph.canvas();
    let canvas_for_listener = canvas.clone();
    let state_for_click = Rc::clone(&state);
    let window_for_click = window.clone();

    let on_click = Closure::wrap(Box::new(move |event: MouseEvent| {
        cancel_scheduled_animation(&window_for_click, &state_for_click);

        let rect = canvas_for_listener.get_bounding_client_rect();
        let scale_x = canvas_for_listener.width() as f64 / rect.width().max(1.0);
        let scale_y = canvas_for_listener.height() as f64 / rect.height().max(1.0);
        let x = (event.client_x() as f64 - rect.left()) * scale_x;
        let y = (event.client_y() as f64 - rect.top()) * scale_y;

        let pitch_class = {
            let state = state_for_click.borrow();
            state.euler_graph.pitch_class_at(x, y)
        };
        if let Some(pitch_class) = pitch_class {
            let _ = state_for_click.borrow_mut().apply_pitch_class(pitch_class);
        }
    }) as Box<dyn FnMut(MouseEvent)>);

    canvas.add_event_listener_with_callback("click", on_click.as_ref().unchecked_ref())?;
    on_click.forget();

    Ok(())
}

pub(super) fn bind_piano_click(
    window: &Window,
    state: Rc<RefCell<AppState>>,
) -> Result<(), JsValue> {
    let canvas = state.borrow().piano.canvas();
    let canvas_for_listener = canvas.clone();
    let state_for_click = Rc::clone(&state);
    let window_for_click = window.clone();

    let on_click = Closure::wrap(Box::new(move |event: MouseEvent| {
        cancel_scheduled_animation(&window_for_click, &state_for_click);
        let (x, y) = canvas_event_position(&canvas_for_listener, &event);
        let semitone = {
            let state = state_for_click.borrow();
            state.piano.note_at(x, y)
        };
        if let Some(semitone) = semitone {
            let _ = state_for_click.borrow_mut().apply_manual_note(semitone);
        }
    }) as Box<dyn FnMut(MouseEvent)>);

    canvas.add_event_listener_with_callback("click", on_click.as_ref().unchecked_ref())?;
    on_click.forget();

    Ok(())
}

pub(super) fn bind_guitar_click(
    window: &Window,
    state: Rc<RefCell<AppState>>,
) -> Result<(), JsValue> {
    let canvas = state.borrow().guitar.canvas();
    let canvas_for_listener = canvas.clone();
    let state_for_click = Rc::clone(&state);
    let window_for_click = window.clone();

    let on_click = Closure::wrap(Box::new(move |event: MouseEvent| {
        cancel_scheduled_animation(&window_for_click, &state_for_click);
        let (x, y) = canvas_event_position(&canvas_for_listener, &event);
        let semitone = {
            let state = state_for_click.borrow();
            state.guitar.note_at(x, y)
        };
        if let Some(semitone) = semitone {
            let _ = state_for_click.borrow_mut().apply_manual_note(semitone);
        }
    }) as Box<dyn FnMut(MouseEvent)>);

    canvas.add_event_listener_with_callback("click", on_click.as_ref().unchecked_ref())?;
    on_click.forget();

    Ok(())
}

fn canvas_event_position(canvas: &HtmlCanvasElement, event: &MouseEvent) -> (f64, f64) {
    let rect = canvas.get_bounding_client_rect();
    let scale_x = canvas.width() as f64 / rect.width().max(1.0);
    let scale_y = canvas.height() as f64 / rect.height().max(1.0);
    (
        (event.client_x() as f64 - rect.left()) * scale_x,
        (event.client_y() as f64 - rect.top()) * scale_y,
    )
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
