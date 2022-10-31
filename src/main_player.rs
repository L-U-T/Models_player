use web_sys::WebGl2RenderingContext;
use yew::prelude::*;
use yew_canvas::{Canvas, WithRander};

mod error;
mod resources;
mod wgpu_state;

#[derive(Clone, PartialEq)]
pub(super) struct Rander {
    pub cursor_to: (i32, i32),
}

use wgpu_state::State;

impl WithRander for Rander {
    fn rand(self, canvas: &web_sys::HtmlCanvasElement) {
        {
            let canvas = canvas.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let _ = State::get_or_init(&canvas).await.unwrap();
            });
        }

        if let Ok(state) = State::get() {
            state
                .display_change(canvas.width(), canvas.height(), self.cursor_to)
                .unwrap();

            // you have to clear the anima_list here,
            // bucause this function will be called after
            // every update, clear the anima_list can
            // prevent repeat push these closures
            state.animation_clear();

            state.animation_push(Box::new(|_| gloo::console::log!("anima!")));
        }
    }
}

#[function_component(MainPlayer)]
pub fn main_player() -> Html {
    let is_hold_state = use_state(|| false);
    let cursor_to_state = use_state(|| (1i32, 0i32));

    let onmousedown = {
        let is_hold_state = is_hold_state.clone();
        Callback::from(move |_| {
            is_hold_state.set(true);
        })
    };

    let onmouseup = {
        let is_hold_state = is_hold_state.clone();
        Callback::from(move |_| {
            is_hold_state.set(false);
        })
    };

    let onmousemove = {
        let cursor_to_state = cursor_to_state.clone();

        Callback::from(move |e: MouseEvent| {
            if *is_hold_state {
                let cursor = (e.screen_x(), e.screen_y());

                cursor_to_state.set((
                    (cursor_to_state.0 + cursor.0) % 360,
                    (cursor_to_state.1 + cursor.1) % 360,
                ));
            }
        })
    };

    let rander = Rander {
        cursor_to: *cursor_to_state,
    };

    html!(
        <div
            {onmousedown}
            {onmouseup}
            {onmousemove}
            style="width: 100%; height: 100%;"
        >
            <Canvas<WebGl2RenderingContext , Rander>
                rander={Box::new(rander)}
                style="height: 100vh; width: 100vw;"
            >
                <h1>
                    {"Sorry, brower u use dont support canvas"}
                </h1>
            </Canvas<WebGl2RenderingContext , Rander>>
        </div>
    )
}
