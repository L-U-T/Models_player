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
                let state = State::get_or_init(&canvas).await.unwrap();

                //render pass per 17 ms
                gloo::timers::callback::Interval::new(17, || {
                    state.anima_pass().unwrap();
                })
                .forget();
            });
        }

        if let Ok(state) = State::get() {
            state
                .display_change(canvas.width(), canvas.height(), self.cursor_to)
                .unwrap();
        }
    }
}

#[function_component(MainPlayer)]
pub fn main_player() -> Html {
    let is_hold_state = use_state(|| false);
    let cursor_move_state = use_state(|| (0i32, 0i32));
    let cursor_to_state = use_state(|| (0i32, 0i32));

    let onmousedown = {
        let is_hold_state = is_hold_state.clone();
        let cursor_move_state = cursor_move_state.clone();
        Callback::from(move |e: MouseEvent| {
            let cursor = (e.screen_x(), e.screen_y());
            cursor_move_state.set(cursor);

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
        let cursor_before = *cursor_move_state.clone();
        let cursor_to_state = cursor_to_state.clone();

        Callback::from(move |e: MouseEvent| {
            if *is_hold_state {
                let cursor = (e.screen_x(), e.screen_y());
                cursor_to_state.set((cursor_before.0 - cursor.0, cursor_before.1 - cursor.1));
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
