use web_sys::WebGl2RenderingContext;
use yew::prelude::*;
use yew_canvas::{Canvas, WithRander};

mod error;
mod resources;
mod wgpu_state;

#[derive(Clone, PartialEq)]
pub(super) struct Rander {
    pub cursor_to: (f32, f32),
    pub wheel_to: f32,
}

use wgpu_state::State;

static mut CANVAS_SIZE: (u32, u32) = (0, 0);

impl WithRander for Rander {
    fn rand(self, canvas: &web_sys::HtmlCanvasElement) {
        let canvas_size = (canvas.width(), canvas.height());

        if unsafe { CANVAS_SIZE != canvas_size } {
            let canvas = canvas.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let state = State::get_or_init(&canvas).await.unwrap();

                state.render().unwrap();
            });

            unsafe { CANVAS_SIZE = canvas_size }
        }

        if let Ok(state) = State::get() {
            state
                .display_change(canvas_size.0, canvas_size.1, self.cursor_to, self.wheel_to)
                .unwrap();
        }
    }
}

const SPEED: f32 = 0.003;

#[function_component(MainPlayer)]
pub fn main_player() -> Html {
    let is_hold_state = use_state(|| false);
    let cursor_state = use_state(|| (0.0, 0.0));
    let cursor_to_state = use_state(|| (-1.0, -0.5));
    let wheel_to_state = use_state(|| 100.0);

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
                let cursor_to = (
                    (cursor_to_state.0 + cursor.0 - cursor_state.0) % 2.0,
                    (cursor_to_state.1 + cursor.1 - cursor_state.1),
                );

                cursor_to_state.set(if cursor_to.1 <= 0.0 && cursor_to.1 >= -1.0 {
                    cursor_to
                } else {
                    *cursor_to_state
                });

                cursor_state.set(cursor);
            }
        })
    };

    let onwheel = {
        let wheel_to_state = wheel_to_state.clone();

        Callback::from(move |e: WheelEvent| {
            wheel_to_state.set((*wheel_to_state + (e.delta_y() as f32)).abs())
        })
    };

    let rander = Rander {
        cursor_to: *cursor_to_state,
        wheel_to: *wheel_to_state,
    };

    html!(
        <div
            {onmousedown}
            {onmouseup}
            {onmousemove}
            {onwheel}
            style="width: 100%; height: 100%;"
        >
            <Canvas<WebGl2RenderingContext , Rander>
                rander={Box::new(rander)}
                style="height: 100vh; width: 100vw;"
            >
                <h1>
                    {"Sorry, browser u use don't support canvas"}
                </h1>
            </Canvas<WebGl2RenderingContext , Rander>>
        </div>
    )
}
