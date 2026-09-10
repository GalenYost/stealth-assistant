#![windows_subsystem = "windows"]

mod app;
mod config;
mod llm;
mod logging;
mod platform;

use app::StealthApp;
use config::AppConfig;
use eframe::egui;

#[tokio::main]
async fn main() -> eframe::Result<()> {
    let config = AppConfig::load();

    match logging::init(&config.log_file_name) {
        Ok(path) => log::info!("log file: {}", path.display()),
        Err(e) => eprintln!("failed to init logging: {e}"),
    }
    log::info!("starting stealth-assistant {}", env!("CARGO_PKG_VERSION"));

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
