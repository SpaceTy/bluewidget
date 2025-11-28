mod bluetooth;
mod config;
mod ui;

use gtk4::prelude::*;
use gtk4::Application;
use ui::window::Window;

fn main() {
    let app = Application::builder()
        .application_id("com.bluewidget")
        .build();

    app.connect_activate(|app| {
        let window = Window::new(app);
        window.window.present();
    });

    app.run();
}