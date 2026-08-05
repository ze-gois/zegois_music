pub mod bind {
    use crate::main::AppState;
    use std::{cell::RefCell, rc::Rc};
    use wasm_bindgen::{JsCast, JsValue, closure::Closure};
    use web_sys::Window;

    pub fn click(window: &Window, state: Rc<RefCell<AppState>>) -> Result<(), JsValue> {
        let stop_button = state.borrow().stop_button.clone();
        let window_for_stop = window.clone();
        let state_for_stop = Rc::clone(&state);

        let on_click = Closure::wrap(Box::new(move || {
            crate::main::cancel_scheduled_animation(&window_for_stop, &state_for_stop);

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
}
