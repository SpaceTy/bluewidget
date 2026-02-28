mod bluetooth;
mod config;
mod ui;

use gtk4::prelude::*;
use gtk4::Application;
use ui::window::Window;
use std::time::Instant;

fn main() {
    let total_start = Instant::now();
    println!("[SPEED] main() started");
    
    let app_build_start = Instant::now();
    let app = Application::builder()
        .application_id("com.bluewidget")
        .build();
    println!("[SPEED] Application::builder().build() took: {:.2?}", app_build_start.elapsed());

    app.connect_activate(|app| {
        let activate_start = Instant::now();
        println!("[SPEED] connect_activate() callback started");

        let window = Window::new(app);
        let window_elapsed = activate_start.elapsed();
        println!("[SPEED] Window::new() took: {:.2?}", window_elapsed);

        let present_start = Instant::now();
        window.present();
        println!("[SPEED] window.present() took: {:.2?}", present_start.elapsed());

        println!("[SPEED] Total activate time: {:.2?}", activate_start.elapsed());
    });

    println!("[SPEED] Starting app.run()...");
    let app_run_start = Instant::now();
    app.run();
    println!("[SPEED] app.run() took: {:.2?}", app_run_start.elapsed());
    println!("[SPEED] Total application runtime: {:.2?}", total_start.elapsed());
}