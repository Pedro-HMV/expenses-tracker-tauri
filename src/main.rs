mod app;

use app::AppData;

fn main() {
    yew::Renderer::<AppData>::new().render();
}
