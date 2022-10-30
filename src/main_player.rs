use web_sys::WebGl2RenderingContext;
use yew::prelude::*;
use yew_canvas::{Canvas, WithRander};

mod error;
mod resources;
mod wgpu_state;

#[derive(Clone, PartialEq)]
pub(super) struct Rander();

use wgpu_state::State;

impl WithRander for Rander {
    fn rand(self, canvas: &web_sys::HtmlCanvasElement) {
        {
            let canvas = canvas.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let state = State::get_or_init(&canvas).await.unwrap();

                //render pass per 17 ms
                gloo::timers::callback::Interval::new(17, || {
                    state.render().unwrap();
                })
                .forget();
            });
        }
    }
}

#[function_component(MainPlayer)]
pub fn main_player() -> Html {
    let rander = Rander();

    html!(
        <Canvas<WebGl2RenderingContext , Rander>
            rander={Box::new(rander)}
            style="height: 100vh; width: 100vw;"
        >
            <h1>
                {"Sorry, brower u use dont support canvas"}
            </h1>
        </Canvas<WebGl2RenderingContext , Rander>>
    )
}
