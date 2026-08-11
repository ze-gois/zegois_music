pub mod bind {
    use crate::entry::AppState;
    use std::{cell::RefCell, rc::Rc};
    use wasm_bindgen::{JsCast, JsValue, closure::Closure};
    pub fn click(state: Rc<RefCell<AppState>>) -> Result<(), JsValue> {
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
}
