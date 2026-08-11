pub mod bind {
    use crate::entry::AppState;
    use std::{cell::RefCell, rc::Rc};
    use wasm_bindgen::{JsCast, JsValue, closure::Closure};
    use web_sys::{MouseEvent, Window};
    pub fn click(window: &Window, state: Rc<RefCell<AppState>>) -> Result<(), JsValue> {
        let canvas = state.borrow().euler_graph.canvas();
        let canvas_for_listener = canvas.clone();
        let state_for_click = Rc::clone(&state);
        let window_for_click = window.clone();

        let on_click = Closure::wrap(Box::new(move |event: MouseEvent| {
            crate::entry::cancel_scheduled_animation(&window_for_click, &state_for_click);

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
}
