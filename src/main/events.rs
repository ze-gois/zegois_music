//! DOM event binding and animation callbacks.
//!
//! Functions in this module attach browser closures to controls and canvases.
//! Hit-testing borrows [`AppState`] only long enough to read the visualizer, then
//! drops that borrow before mutating state; this avoids runtime `RefCell` panics
//! in WASM click handlers.
use std::{cell::RefCell, rc::Rc};

use wasm_bindgen::{JsCast, closure::Closure};
use web_sys::Window;

use super::state::AppState;

/// Create the shared `requestAnimationFrame` closure used during playback.
pub fn create_animation_loop(
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
