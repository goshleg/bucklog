use bucklog::app;
use bucklog::app::config;
use egui::{Vec2, ViewportBuilder};

fn main() {
    let config = config::Settings::load_configuration();
    let native_options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_always_on_top()
            .with_inner_size(Vec2::new(720.0, 480.0)),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        native_options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc, config)))),
    )
    .expect("Failed to run app");
}
