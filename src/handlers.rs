use js_sys::{Array, Number, Reflect};
use wasm_bindgen::prelude::*;

use crate::js_helpers::{delete_key, get, get_f64, get_id, get_str, set, to_id, vec_to_array};

fn ensure_mesh(s: &JsValue) {
    let stype = get_str(s, "type");
    if stype == "path" || stype == "dot" {
        return;
    }
    let has_mesh = get(s, "mesh");
    if !has_mesh.is_undefined() && !has_mesh.is_null() {
        return;
    }
    let x1 = get_f64(s, "x1");
    let y1 = get_f64(s, "y1");
    let x2 = get_f64(s, "x2");
    let y2 = get_f64(s, "y2");
    let sz = get_f64(s, "size");
    let mesh = crate::wasm::generate_shape_mesh(&stype, x1, y1, x2, y2, sz);
    set(s, "mesh", &vec_to_array(&mesh));
}

fn add_stroke_to_array(arr: &Array, s: &JsValue) {
    let sid = get_str(s, "id");
    if sid.is_empty() {
        arr.push(s);
        return;
    }
    for i in 0..arr.length() {
        let existing = arr.get(i);
        if get_str(&existing, "id") == sid {
            arr.set(i, s.clone());
            return;
        }
    }
    arr.push(s);
}

#[wasm_bindgen]
pub fn ws_init(msg: &JsValue, remote_cursors: &JsValue, strokes: &JsValue) {
    let my_id = get_id(msg, "id");

    let users = get(msg, "users");
    if let Some(arr) = users.dyn_ref::<Array>() {
        for i in 0..arr.length() {
            let u = arr.get(i);
            let uid = get_id(&u, "id");
            if uid.is_empty() || uid == my_id {
                continue;
            }
            let color = get_str(&u, "color");
            let obj = js_sys::Object::new();
            set(&obj, "color", &color.into());
            set(&obj, "x", &Number::from(-999.0));
            set(&obj, "y", &Number::from(-999.0));
            set(&obj, "rx", &Number::from(-999.0));
            set(&obj, "ry", &Number::from(-999.0));
            set(&obj, "tx", &Number::from(-999.0));
            set(&obj, "ty", &Number::from(-999.0));
            let _ = Reflect::set(remote_cursors, &uid.into(), &obj);
        }
    }

    let stored = get(msg, "strokes");
    if let Some(arr) = stored.dyn_ref::<Array>() {
        for i in 0..arr.length() {
            let entry = arr.get(i);
            let s = get(&entry, "stroke");
            let s = if s.is_undefined() || s.is_null() {
                entry.clone()
            } else {
                s
            };
            let user_id = get_id(&entry, "user_id");
            if !user_id.is_empty() {
                set(&s, "userId", &user_id.into());
            }
            let sid = get_str(&s, "id");
            if sid.is_empty() {
                let ts = js_sys::Date::now() as u64;
                let r: u32 = (js_sys::Math::random() * 1_000_000.0) as u32;
                let new_id = format!("srv_{:x}_{:x}", ts, r);
                set(&s, "id", &new_id.into());
            }
            ensure_mesh(&s);
            if let Some(strokes_arr) = strokes.dyn_ref::<Array>() {
                strokes_arr.push(&s);
            }
        }
    }
}

#[wasm_bindgen]
pub fn ws_join(msg: &JsValue, remote_cursors: &JsValue) {
    let uid = get_id(msg, "id");
    if uid.is_empty() {
        return;
    }
    let color = get_str(msg, "color");
    let obj = js_sys::Object::new();
    set(&obj, "color", &color.into());
    set(&obj, "x", &Number::from(-999.0));
    set(&obj, "y", &Number::from(-999.0));
    set(&obj, "rx", &Number::from(-999.0));
    set(&obj, "ry", &Number::from(-999.0));
    set(&obj, "tx", &Number::from(-999.0));
    set(&obj, "ty", &Number::from(-999.0));
    let _ = Reflect::set(remote_cursors, &uid.into(), &obj);
}

#[wasm_bindgen]
pub fn ws_leave(msg: &JsValue, remote_cursors: &JsValue, strokes: &JsValue) {
    let uid = get_id(msg, "id");
    if uid.is_empty() {
        return;
    }
    delete_key(remote_cursors, &uid);

    if let Some(arr) = strokes.dyn_ref::<Array>() {
        let mut i = arr.length() as i32 - 1;
        while i >= 0 {
            let s = arr.get(i as u32);
            if get_str(&s, "userId") == uid {
                arr.splice_many(i as u32, 1, &[]);
            }
            i -= 1;
        }
    }
}

#[wasm_bindgen]
pub fn ws_stroke_end(
    msg: &JsValue,
    live_strokes: &JsValue,
    live_shapes: &JsValue,
    strokes: &JsValue,
) {
    let data = get(msg, "data");
    if data.is_undefined() {
        return;
    }

    let sid = get_str(&data, "id");
    if !sid.is_empty() {
        delete_key(live_strokes, &sid);
        delete_key(live_shapes, &sid);
    }

    let stroke = get(&data, "stroke");
    let s = if stroke.is_undefined() || stroke.is_null() {
        data.clone()
    } else {
        stroke
    };

    let user_id = get_id(msg, "id");
    if !user_id.is_empty() {
        set(&s, "userId", &user_id.into());
    }
    if sid.is_empty() {
        let did = get_str(&data, "id");
        if !did.is_empty() {
            set(&s, "id", &did.into());
        }
    }

    ensure_mesh(&s);

    if let Some(arr) = strokes.dyn_ref::<Array>() {
        add_stroke_to_array(arr, &s);
    }
}

#[wasm_bindgen]
pub fn ws_stroke_start(msg: &JsValue, live_strokes: &JsValue) {
    let data = get(msg, "data");
    if data.is_undefined() {
        return;
    }
    let sid = get_str(&data, "id");
    if sid.is_empty() {
        return;
    }
    let color = get_str(&data, "color");
    let size = get_f64(&data, "size");
    let obj = js_sys::Object::new();
    set(&obj, "mesh", &JsValue::null());
    set(&obj, "color", &color.into());
    set(&obj, "size", &Number::from(size));
    let _ = Reflect::set(live_strokes, &sid.into(), &obj);
}

#[wasm_bindgen]
pub fn ws_stroke_update(msg: &JsValue, live_strokes: &JsValue) {
    let data = get(msg, "data");
    if data.is_undefined() {
        return;
    }
    let sid = get_str(&data, "id");
    if sid.is_empty() {
        return;
    }
    let s = Reflect::get(live_strokes, &sid.into()).unwrap_or(JsValue::UNDEFINED);
    if s.is_undefined() || s.is_null() {
        return;
    }
    let mesh = get(&data, "mesh");
    if !mesh.is_undefined() && !mesh.is_null() {
        set(&s, "mesh", &mesh);
    }
}

#[wasm_bindgen]
pub fn ws_shape_update(msg: &JsValue, live_shapes: &JsValue) -> bool {
    let data = get(msg, "data");
    if data.is_undefined() {
        return false;
    }
    let shape = get(&data, "shape");
    if shape.is_undefined() {
        return false;
    }
    let stype = get_str(&shape, "type");
    if stype == "path" || stype == "dot" {
        return false;
    }
    let sid = get_str(&data, "id");
    if sid.is_empty() {
        return false;
    }

    let existing = Reflect::get(live_shapes, &sid.clone().into()).unwrap_or(JsValue::UNDEFINED);
    let is_first = existing.is_undefined() || existing.is_null();

    if is_first {
        let obj = js_sys::Object::new();
        set(&obj, "type", &stype.into());
        let color = get_str(&shape, "color");
        set(&obj, "color", &color.into());
        let size = get_f64(&shape, "size");
        set(&obj, "size", &Number::from(size));
        let _ = Reflect::set(live_shapes, &sid.clone().into(), &obj);
    }

    let s_ref = Reflect::get(live_shapes, &sid.into()).unwrap_or(JsValue::UNDEFINED);
    if s_ref.is_undefined() {
        return false;
    }

    let x1 = get_f64(&shape, "x1");
    let y1 = get_f64(&shape, "y1");
    let x2 = get_f64(&shape, "x2");
    let y2 = get_f64(&shape, "y2");
    let color = get_str(&shape, "color");
    let size = get_f64(&shape, "size");

    set(&s_ref, "tx1", &Number::from(x1));
    set(&s_ref, "ty1", &Number::from(y1));
    set(&s_ref, "tx2", &Number::from(x2));
    set(&s_ref, "ty2", &Number::from(y2));
    set(&s_ref, "color", &color.into());
    set(&s_ref, "size", &Number::from(size));

    if is_first {
        set(&s_ref, "rx1", &Number::from(x1));
        set(&s_ref, "ry1", &Number::from(y1));
        set(&s_ref, "rx2", &Number::from(x2));
        set(&s_ref, "ry2", &Number::from(y2));
        let st = get_str(&s_ref, "type");
        let sz = get_f64(&s_ref, "size");
        let mesh = crate::wasm::generate_shape_mesh(&st, x1, y1, x2, y2, sz);
        set(&s_ref, "mesh", &vec_to_array(&mesh));
        return true;
    }
    false
}

#[wasm_bindgen]
pub fn ws_cursor(msg: &JsValue, remote_cursors: &JsValue) {
    let uid = get_id(msg, "id");
    if uid.is_empty() {
        return;
    }
    let c = Reflect::get(remote_cursors, &uid.into()).unwrap_or(JsValue::UNDEFINED);
    if c.is_undefined() || c.is_null() {
        return;
    }
    let x = get_f64(msg, "x");
    let y = get_f64(msg, "y");
    let tx = get_f64(&c, "tx");
    if tx == -999.0 {
        set(&c, "rx", &Number::from(x));
        set(&c, "ry", &Number::from(y));
    }
    set(&c, "tx", &Number::from(x));
    set(&c, "ty", &Number::from(y));
}

#[wasm_bindgen]
pub fn ws_erase(msg: &JsValue, strokes: &JsValue) {
    let ids_val = get(msg, "ids");
    let ids = match ids_val.dyn_ref::<Array>() {
        Some(a) => a,
        None => return,
    };
    if ids.length() == 0 {
        return;
    }
    let owners_val = get(msg, "owners");
    let owners = owners_val.dyn_ref::<Array>();

    let arr = match strokes.dyn_ref::<Array>() {
        Some(a) => a,
        None => return,
    };

    for i in 0..ids.length() {
        let id = ids.get(i).as_string().unwrap_or_default();
        if id.is_empty() {
            continue;
        }
        let owner_id = owners.map(|o| to_id(&o.get(i))).filter(|s| !s.is_empty());

        let mut found = false;
        let mut j = arr.length() as i32 - 1;
        while j >= 0 {
            let s = arr.get(j as u32);
            if get_str(&s, "id") == id {
                arr.splice_many(j as u32, 1, &[]);
                found = true;
                break;
            }
            j -= 1;
        }
        if !found && let Some(oid) = owner_id {
            let mut k = arr.length() as i32 - 1;
            while k >= 0 {
                let s = arr.get(k as u32);
                if get_str(&s, "userId") == oid {
                    arr.splice_many(k as u32, 1, &[]);
                    break;
                }
                k -= 1;
            }
        }
    }
}

pub struct Bounds {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
}

#[wasm_bindgen]
pub fn transform_move(sel: &JsValue, dx: f64, dy: f64) {
    let stype = get_str(sel, "type");
    if stype == "dot" {
        let x = get_f64(sel, "x");
        let y = get_f64(sel, "y");
        set(sel, "x", &Number::from(x + dx));
        set(sel, "y", &Number::from(y + dy));
    } else if stype == "path" {
        let data = get(sel, "data");
        if let Some(arr) = data.dyn_ref::<Array>() {
            for i in 0..arr.length() {
                if i % 3 == 2 {
                    continue;
                }
                let v = arr.get(i).as_f64().unwrap_or(0.0);
                let new_v = v + if i % 3 == 0 { dx } else { dy };
                arr.set(i, JsValue::from(new_v));
            }
        }
    } else if stype == "rect" || stype == "circle" || stype == "line" || stype == "arrow" {
        let x1 = get_f64(sel, "x1");
        let y1 = get_f64(sel, "y1");
        let x2 = get_f64(sel, "x2");
        let y2 = get_f64(sel, "y2");
        set(sel, "x1", &Number::from(x1 + dx));
        set(sel, "y1", &Number::from(y1 + dy));
        set(sel, "x2", &Number::from(x2 + dx));
        set(sel, "y2", &Number::from(y2 + dy));
    }
}

#[allow(clippy::too_many_arguments)]
#[wasm_bindgen]
pub fn transform_resize(
    sel: &JsValue,
    handle: &str,
    x: f64,
    y: f64,
    bx1: f64,
    by1: f64,
    bx2: f64,
    by2: f64,
) {
    let b = Bounds {
        x1: bx1,
        y1: by1,
        x2: bx2,
        y2: by2,
    };
    let nx1 = if handle.contains('w') { x } else { b.x1 };
    let ny1 = if handle.contains('n') { y } else { b.y1 };
    let nx2 = if handle.contains('e') { x } else { b.x2 };
    let ny2 = if handle.contains('s') { y } else { b.y2 };
    set(sel, "x1", &Number::from(nx1));
    set(sel, "y1", &Number::from(ny1));
    set(sel, "x2", &Number::from(nx2));
    set(sel, "y2", &Number::from(ny2));
}

#[wasm_bindgen]
pub fn regenerate_mesh(sel: &JsValue) {
    let stype = get_str(sel, "type");
    if stype == "path" {
        let data = get(sel, "data");
        if let Some(arr) = data.dyn_ref::<Array>() {
            let mut cl: Vec<[f64; 3]> = Vec::new();
            for i in (0..arr.length()).step_by(3) {
                let x = arr.get(i).as_f64().unwrap_or(0.0);
                let y = arr.get(i + 1).as_f64().unwrap_or(0.0);
                let w = arr.get(i + 2).as_f64().unwrap_or(0.0);
                cl.push([x, y, w]);
            }
            let mesh_data = crate::geometry::generate_mesh(&cl, 8);
            set(sel, "mesh", &vec_to_array(&mesh_data));
        }
    } else if stype == "rect" || stype == "circle" || stype == "line" || stype == "arrow" {
        let x1 = get_f64(sel, "x1");
        let y1 = get_f64(sel, "y1");
        let x2 = get_f64(sel, "x2");
        let y2 = get_f64(sel, "y2");
        let sz = get_f64(sel, "size");
        let mesh = crate::wasm::generate_shape_mesh(&stype, x1, y1, x2, y2, sz);
        set(sel, "mesh", &vec_to_array(&mesh));
    }
}
