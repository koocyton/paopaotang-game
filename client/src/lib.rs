mod game;
mod input;
mod network;
mod renderer;
mod sprites;

use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use game::Game;

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    console_error_panic_hook_set();

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document
        .get_element_by_id("game-canvas")
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()?;

    canvas.set_width(shared::constants::CANVAS_WIDTH);
    canvas.set_height(shared::constants::CANVAS_HEIGHT + 30);

    let location = window.location();
    let host = location.host().unwrap_or_else(|_| "localhost:3000".into());
    let protocol = location.protocol().unwrap_or_else(|_| "http:".into());
    let ws_protocol = if protocol == "https:" { "wss" } else { "ws" };
    let ws_url = format!("{}://{}/ws", ws_protocol, host);

    let game = Rc::new(RefCell::new(Game::new(&canvas, &ws_url)));

    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();

    let game_loop = game.clone();
    *g.borrow_mut() = Some(Closure::new(move || {
        {
            let mut g = game_loop.borrow_mut();
            g.update();
            g.render();
        }
        request_animation_frame(f.borrow().as_ref().unwrap());
    }));

    request_animation_frame(g.borrow().as_ref().unwrap());

    Ok(())
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    web_sys::window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .unwrap();
}

fn console_error_panic_hook_set() {
    std::panic::set_hook(Box::new(|info| {
        web_sys::console::error_1(&format!("PANIC: {}", info).into());
    }));
}
