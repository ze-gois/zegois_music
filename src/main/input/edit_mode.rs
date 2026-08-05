pub mod group {

    pub mod bind {
        use crate::main::AppState;
        use crate::main::state::EditMode;
        use std::{cell::RefCell, rc::Rc};
        use wasm_bindgen::{JsCast, JsValue, closure::Closure};
        use web_sys::HtmlInputElement;
        pub fn click(
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
    }
}

pub mod bind {
    use crate::main::AppState;
    use std::{cell::RefCell, rc::Rc};
    use wasm_bindgen::JsValue;
    use web_sys::Document;

    use crate::main::state::EditMode;
    pub fn click(document: &Document, state: Rc<RefCell<AppState>>) -> Result<(), JsValue> {
        super::group::bind::click(
            webspace::dom::element_by_id(document, "replaceMode")?,
            Rc::clone(&state),
            EditMode::Replace,
        )?;
        super::group::bind::click(
            webspace::dom::element_by_id(document, "insertMode")?,
            Rc::clone(&state),
            EditMode::Insert,
        )?;
        super::group::bind::click(
            webspace::dom::element_by_id(document, "appendMode")?,
            state,
            EditMode::Append,
        )?;

        Ok(())
    }
}
