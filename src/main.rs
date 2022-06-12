use eframe::egui;
use log::{info, trace, warn};
use pretty_env_logger;

fn main() {
    pretty_env_logger::init();
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Arcana Launcher",
        options,
        Box::new(|_cc| Box::new(LauncherApp::default())),
    );
}

struct LauncherApp;

impl Default for LauncherApp {
    fn default() -> Self {
        Self
    }
}

impl eframe::App for LauncherApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Welcome!");
            if ui.button("Play").clicked() {
                info!("You haven't got that working yet.");
            }
        });
    }
}
