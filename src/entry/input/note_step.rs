pub mod bind {
    use crate::entry::AppState;
    use std::{cell::RefCell, rc::Rc};
    use wasm_bindgen::{JsCast, JsValue, closure::Closure};

    pub fn click(state: Rc<RefCell<AppState>>) -> Result<(), JsValue> {
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
}
