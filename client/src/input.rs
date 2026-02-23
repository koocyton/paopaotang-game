use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

pub struct InputState {
    keys_down: Rc<RefCell<HashSet<String>>>,
    keys_just_pressed: Rc<RefCell<HashSet<String>>>,
}

impl InputState {
    pub fn new() -> Self {
        let keys_down = Rc::new(RefCell::new(HashSet::new()));
        let keys_just_pressed = Rc::new(RefCell::new(HashSet::new()));

        let window = web_sys::window().unwrap();

        {
            let kd = keys_down.clone();
            let kjp = keys_just_pressed.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::KeyboardEvent| {
                let key = event.key();
                if !kd.borrow().contains(&key) {
                    kjp.borrow_mut().insert(key.clone());
                }
                kd.borrow_mut().insert(key);
                event.prevent_default();
            });
            window
                .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
                .unwrap();
            closure.forget();
        }

        {
            let kd = keys_down.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::KeyboardEvent| {
                kd.borrow_mut().remove(&event.key());
            });
            window
                .add_event_listener_with_callback("keyup", closure.as_ref().unchecked_ref())
                .unwrap();
            closure.forget();
        }

        InputState {
            keys_down,
            keys_just_pressed,
        }
    }

    pub fn is_key_down(&self, key: &str) -> bool {
        self.keys_down.borrow().contains(key)
    }

    pub fn is_key_just_pressed(&self, key: &str) -> bool {
        self.keys_just_pressed.borrow().contains(key)
    }

    pub fn clear_frame(&self) {
        self.keys_just_pressed.borrow_mut().clear();
    }

    pub fn get_movement(&self) -> (f64, f64) {
        let mut dx = 0.0;
        let mut dy = 0.0;
        if self.is_key_down("ArrowUp") || self.is_key_down("w") || self.is_key_down("W") {
            dy -= 1.0;
        }
        if self.is_key_down("ArrowDown") || self.is_key_down("s") || self.is_key_down("S") {
            dy += 1.0;
        }
        if self.is_key_down("ArrowLeft") || self.is_key_down("a") || self.is_key_down("A") {
            dx -= 1.0;
        }
        if self.is_key_down("ArrowRight") || self.is_key_down("d") || self.is_key_down("D") {
            dx += 1.0;
        }
        (dx, dy)
    }

    pub fn wants_bomb(&self) -> bool {
        self.is_key_just_pressed(" ")
    }
}
