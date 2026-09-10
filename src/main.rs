#![windows_subsystem = "windows"]

mod app;
mod config;
mod llm;
mod platform;

use app::StealthApp;
use eframe::egui;

#[tokio::main]
async fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_transparent(true)
            .with_always_on_top()
            .with_decorations(false)
            .with_inner_size([480.0, 650.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Stealth Assistant",
        options,
        Box::new(|cc| Ok(Box::new(StealthApp::new(cc)))),
    )
}
