mod dom;
mod events;
mod music;
mod state;
mod ui;

use std::{cell::RefCell, rc::Rc};

use wasm_bindgen::{JsValue, prelude::*};
use web_sys::window;

use state::AppState;
use ui::APP_HTML;

#[wasm_bindgen]
pub fn start_app() -> Result<(), JsValue> {
    let window = window().ok_or_else(|| JsValue::from_str("window is not available"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("document is not available"))?;

    let root = document
        .get_element_by_id("app")
        .ok_or_else(|| JsValue::from_str("#app root element was not found"))?;
    root.set_inner_html(APP_HTML);

    let state = Rc::new(RefCell::new(AppState::new(&document)?));
    state.borrow().redraw_all_idle()?;

    let animation = events::create_animation_loop(&window, Rc::clone(&state));
    state.borrow().update_note_step_ui();
    state.borrow().update_melody_status();
    events::bind_bpm_input(Rc::clone(&state))?;
    events::bind_play_button(&window, Rc::clone(&state), Rc::clone(&animation))?;
    events::bind_stop_button(&window, Rc::clone(&state))?;
    events::bind_reset_button(Rc::clone(&state))?;
    events::bind_walk_button(Rc::clone(&state))?;
    events::bind_clear_button(Rc::clone(&state))?;
    events::bind_note_step_input(Rc::clone(&state))?;
    events::bind_edit_mode_inputs(&document, Rc::clone(&state))?;
    events::bind_euler_click(&window, Rc::clone(&state))?;
    events::bind_piano_click(&window, Rc::clone(&state))?;
    events::bind_guitar_click(&window, Rc::clone(&state))?;

    Ok(())
}
