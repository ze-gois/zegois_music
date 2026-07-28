use wasm_bindgen::{JsCast, JsValue};
use web_sys::Document;

pub(super) fn element_by_id<T>(document: &Document, id: &str) -> Result<T, JsValue>
where
    T: JsCast,
{
    document
        .get_element_by_id(id)
        .ok_or_else(|| JsValue::from_str(&format!("#{id} element was not found")))?
        .dyn_into::<T>()
        .map_err(|_| JsValue::from_str(&format!("#{id} element has the wrong type")))
}
