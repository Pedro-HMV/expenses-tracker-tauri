mod app;

use app::App;
use app::AppData;

fn main() {
    yew::Renderer::<App>::new().render();
    yew::Renderer::<AppData>::new().render();
}
