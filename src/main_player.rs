use std::f32::consts::PI;

use web_sys::WebGl2RenderingContext;
use yew::prelude::*;
use yew_canvas::{Canvas, WithRander};

mod error;
mod resources;
mod wgpu_state;

#[derive(Clone, PartialEq)]
pub(super) struct Rander {
    pub cursor_to: (f32, f32),
}

use wgpu_state::State;

impl WithRander for Rander {
    fn rand(self, canvas: &web_sys::HtmlCanvasElement) {
        if let Ok(state) = State::get() {
            state
                .display_change(canvas.width(), canvas.height(), self.cursor_to)
                .unwrap();
        } else {
            let canvas = canvas.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let state = State::get_or_init(&canvas).await.unwrap();

                state.animation_insert("log test".to_owned(), Box::new(|_| ()));

                state.render().unwrap();
            });
        }
    }
}

const SPEED:f32 = 0.001;

#[function_component(MainPlayer)]
pub fn main_player() -> Html {
    let is_hold_state = use_state(|| false);
    let cursor_state = use_state(|| (0.0, 0.0));
    let cursor_to_state = use_state(|| (-1.0, -0.5));

    let onmousedown = {
        let cursor_state = cursor_state.clone();
        let is_hold_state = is_hold_state.clone();
        Callback::from(move |e: MouseEvent| {
            let cursor = (e.screen_x() as f32 * SPEED, e.screen_y() as f32 * SPEED);

            cursor_state.set(cursor);
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
                let cursor = (e.screen_x() as f32 * SPEED, e.screen_y() as f32 * SPEED);

                cursor_to_state.set((
                    (cursor_to_state.0 + cursor.0 - cursor_state.0) % 360.0,
                    (cursor_to_state.1 + cursor.1 - cursor_state.1) % 360.0,
                ));

                cursor_state.set(cursor);
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
