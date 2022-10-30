use yew::prelude::*;

mod main_player;

#[function_component(App)]
fn app() -> Html {
    html!(
        //Where the graphic show
        <main_player::MainPlayer />
    )
}

fn main() {
    yew::start_app::<App>();
}
