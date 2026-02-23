use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::WebSocket;

use shared::protocol::{ClientMsg, ServerMsg};

pub struct Network {
    ws: WebSocket,
    messages: Rc<RefCell<VecDeque<ServerMsg>>>,
    connected: Rc<RefCell<bool>>,
}

impl Network {
    pub fn new(url: &str) -> Self {
        let ws = WebSocket::new(url).expect("Failed to create WebSocket");
        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

        let messages = Rc::new(RefCell::new(VecDeque::new()));
        let connected = Rc::new(RefCell::new(false));

        {
            let connected = connected.clone();
            let onopen = Closure::<dyn FnMut()>::new(move || {
                *connected.borrow_mut() = true;
                web_sys::console::log_1(&"WebSocket connected".into());
            });
            ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
            onopen.forget();
        }

        {
            let messages = messages.clone();
            let onmessage =
                Closure::<dyn FnMut(_)>::new(move |event: web_sys::MessageEvent| {
                    if let Ok(text) = event.data().dyn_into::<js_sys::JsString>() {
                        let s: String = text.into();
                        if let Ok(msg) = serde_json::from_str::<ServerMsg>(&s) {
                            messages.borrow_mut().push_back(msg);
                        }
                    }
                });
            ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
            onmessage.forget();
        }

        {
            let connected = connected.clone();
            let onclose = Closure::<dyn FnMut()>::new(move || {
                *connected.borrow_mut() = false;
                web_sys::console::log_1(&"WebSocket disconnected".into());
            });
            ws.set_onclose(Some(onclose.as_ref().unchecked_ref()));
            onclose.forget();
        }

        Network {
            ws,
            messages,
            connected,
        }
    }

    pub fn is_connected(&self) -> bool {
        *self.connected.borrow()
    }

    pub fn send(&self, msg: &ClientMsg) {
        if let Ok(json) = serde_json::to_string(msg) {
            let _ = self.ws.send_with_str(&json);
        }
    }

    pub fn poll(&self) -> Vec<ServerMsg> {
        let mut msgs = self.messages.borrow_mut();
        msgs.drain(..).collect()
    }
}
