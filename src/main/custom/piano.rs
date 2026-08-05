pub mod bind {
    use crate::main::AppState;
    use std::{cell::RefCell, rc::Rc};
    use wasm_bindgen::{JsCast, JsValue, closure::Closure};
    use web_sys::{MouseEvent, Window};

    pub fn click(window: &Window, state: Rc<RefCell<AppState>>) -> Result<(), JsValue> {
        let canvas = state.borrow().piano.canvas();
        let canvas_for_listener = canvas.clone();
        let state_for_click = Rc::clone(&state);
        let window_for_click = window.clone();

        let on_click = Closure::wrap(Box::new(move |event: MouseEvent| {
            crate::main::cancel_scheduled_animation(&window_for_click, &state_for_click);
            let (x, y) = webspace::canvas::mouse_position(&canvas_for_listener, &event);
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
}
