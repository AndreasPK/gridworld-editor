mod dnaparser;

use fltk::{app, prelude::*, window::Window};

fn main() {
    let app = app::App::default();

    let mut wind = Window::default()
        .with_size(800, 600)
        .with_label("Gridworld Editor");

    wind.end();
    wind.show();

    app.run().expect("FLTK event loop failed");
}
