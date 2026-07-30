use js_sys::{Array, Number, Reflect};
use wasm_bindgen::prelude::*;

pub fn set(obj: &JsValue, key: &str, val: &JsValue) {
    let _ = Reflect::set(obj, &key.into(), val);
}

pub fn get(obj: &JsValue, key: &str) -> JsValue {
    Reflect::get(obj, &key.into()).unwrap_or(JsValue::UNDEFINED)
}

pub fn get_f64(obj: &JsValue, key: &str) -> f64 {
    get(obj, key).as_f64().unwrap_or(0.0)
}

pub fn get_str(obj: &JsValue, key: &str) -> String {
    get(obj, key).as_string().unwrap_or_default()
}

pub fn get_id(obj: &JsValue, key: &str) -> String {
    let v = get(obj, key);
    v.as_string()
        .or_else(|| v.as_f64().map(|n| n.to_string()))
        .unwrap_or_default()
}

pub fn get_keys(obj: &JsValue) -> Array {
    Reflect::own_keys(obj).unwrap_or_else(|_| Array::new())
}

pub fn delete_key(obj: &JsValue, key: &str) {
    if let Some(o) = obj.dyn_ref::<js_sys::Object>() {
        let _ = Reflect::delete_property(o, &key.into());
    }
}

pub fn to_id(v: &JsValue) -> String {
    v.as_string()
        .or_else(|| v.as_f64().map(|n| n.to_string()))
        .unwrap_or_default()
}

pub fn vec_to_array(v: &[f64]) -> Array {
    let arr = Array::new();
    for &x in v {
        arr.push(&Number::from(x));
    }
    arr
}
