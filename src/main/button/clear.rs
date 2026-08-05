pub mod bind {
    use crate::main::AppState;
    use std::{cell::RefCell, rc::Rc};
    use wasm_bindgen::{JsCast, JsValue, closure::Closure};

    pub fn click(state: Rc<RefCell<AppState>>) -> Result<(), JsValue> {
        let button = state.borrow().clear_button.clone();
        let state_for_click = Rc::clone(&state);

        let on_click = Closure::wrap(Box::new(move || {
            let _ = state_for_click.borrow_mut().clear_melody();
        }) as Box<dyn FnMut()>);

        button.add_event_listener_with_callback("click", on_click.as_ref().unchecked_ref())?;
        on_click.forget();

        Ok(())
    }
}
