use eframe::egui;
use log::{info, trace, warn};
use octocrab::{self, models::repos::Release};
use pretty_env_logger;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();
    let options = eframe::NativeOptions::default();

    let mut app = LauncherApp::new().await?;

    eframe::run_native("Arcana Launcher", options, Box::new(|_cc| Box::new(app)));
}

struct LauncherApp {
    latest_twelve_knights: Release,
}

impl LauncherApp {
    async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let release = Self::get_latest_twelve_knights().await?;
        Ok(Self {
            latest_twelve_knights: release,
        })
    }

    async fn get_latest_twelve_knights() -> Result<Release, Box<dyn std::error::Error>> {
        let release = octocrab::instance()
            .repos("AlaricTheFool", "Twelve-Knights-Vigil")
            .releases()
            .get_by_tag("latest")
            .await?;
        Ok(release)
    }
}

impl eframe::App for LauncherApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Welcome!");

            ui.label(format!(
                "Last Update: {}",
                self.latest_twelve_knights.published_at.unwrap()
            ));
            if ui.button("Play").clicked() {}
        });
    }
}
