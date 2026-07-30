use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{CloseEvent, Event, MessageEvent, WebSocket};

#[wasm_bindgen]
pub struct WsClient {
    ws: Option<WebSocket>,
}

#[wasm_bindgen]
impl WsClient {
    pub fn new() -> WsClient {
        WsClient { ws: None }
    }

    pub fn connect(
        &mut self,
        url: &str,
        room: &str,
        on_msg: &js_sys::Function,
        on_status: &js_sys::Function,
    ) {
        if self.ws.is_some() {
            return;
        }

        let ws = match WebSocket::new(url) {
            Ok(ws) => ws,
            Err(e) => {
                let _ = on_status.call1(
                    &JsValue::null(),
                    &JsValue::from_str(&format!("error: {e:?}")),
                );
                return;
            }
        };

        let ws_for_open = ws.clone();
        let room_owned = room.to_string();
        let on_open = Closure::wrap(Box::new(move || {
            let join = format!(r#"{{"type":"join","room":"{}"}}"#, room_owned);
            let _ = ws_for_open.send_with_str(&join);
        }) as Box<dyn FnMut()>);
        ws.set_onopen(Some(on_open.as_ref().unchecked_ref()));
        on_open.forget();

        let msg_fn = on_msg.clone();
        let on_message = Closure::wrap(Box::new(move |e: MessageEvent| {
            if let Some(text) = e.data().as_string() {
                let _ = msg_fn.call1(&JsValue::null(), &JsValue::from_str(&text));
            }
        }) as Box<dyn FnMut(MessageEvent)>);
        ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
        on_message.forget();

        let status_fn = on_status.clone();
        let on_close = Closure::wrap(Box::new(move |_: CloseEvent| {
            let _ = status_fn.call1(&JsValue::null(), &JsValue::from_str("disconnected"));
        }) as Box<dyn FnMut(CloseEvent)>);
        ws.set_onclose(Some(on_close.as_ref().unchecked_ref()));
        on_close.forget();

        let on_error = Closure::wrap(Box::new(move |_: Event| {
            // WebSocket errors are usually followed by onclose
        }) as Box<dyn FnMut(web_sys::Event)>);
        ws.set_onerror(Some(on_error.as_ref().unchecked_ref()));
        on_error.forget();

        let _ = on_status.call1(&JsValue::null(), &JsValue::from_str("connecting"));

        self.ws = Some(ws);
    }

    pub fn send(&self, json: &str) {
        if let Some(ref ws) = self.ws {
            let _ = ws.send_with_str(json);
        }
    }

    pub fn disconnect(&mut self) {
        if let Some(ws) = self.ws.take() {
            let _ = ws.close();
        }
    }

    pub fn is_connected(&self) -> bool {
        self.ws.is_some()
    }
}

impl Default for WsClient {
    fn default() -> Self {
        Self::new()
    }
}
