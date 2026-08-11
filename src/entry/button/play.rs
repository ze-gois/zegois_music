pub mod bind {
    use crate::entry::AppState;
    use std::{cell::RefCell, rc::Rc};
    use wasm_bindgen::{JsCast, JsValue, closure::Closure};
    use web_sys::Window;

    pub fn click(
        window: &Window,
        state: Rc<RefCell<AppState>>,
        animation: Rc<RefCell<Option<Closure<dyn FnMut()>>>>,
    ) -> Result<(), JsValue> {
        let play_button = state.borrow().play_button.clone();
        let window_for_play = window.clone();
        let state_for_play = Rc::clone(&state);
        let animation_for_play = Rc::clone(&animation);

        let on_click = Closure::wrap(Box::new(move || {
            crate::entry::cancel_scheduled_animation(&window_for_play, &state_for_play);

            let play_result = state_for_play.borrow_mut().play();
            match play_result {
                Ok(()) => crate::entry::request_next_frame(
                    &window_for_play,
                    &state_for_play,
                    &animation_for_play,
                ),
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
}
