use eframe::egui;
use log::{error, info, trace, warn};
use octocrab::{self, models::repos::Release};
use pretty_env_logger;
use reqwest::Url;
use std::fs::DirBuilder;
use std::fs::File;
use std::io::copy;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();
    create_data_dir();
    let options = eframe::NativeOptions::default();

    let mut app = LauncherApp::new().await?;

    eframe::run_native("Arcana Launcher", options, Box::new(|_cc| Box::new(app)));
}

fn create_data_dir() {
    let mut dir_builder = DirBuilder::new();
    dir_builder.recursive(true);
    dir_builder.create(get_data_dir().as_path()).unwrap();
}

fn get_data_dir() -> PathBuf {
    let mut path = dirs::data_local_dir().unwrap();

    path.push("arcana-launcher");

    path
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
            if ui.button("Download").clicked() {
                trace!("Finding correct download link.");
                if let Some(asset) = self
                    .latest_twelve_knights
                    .assets
                    .iter()
                    .find(|asset| asset.name == "linux-latest.zip".to_string())
                {
                    trace!(
                        "Found asset with download link: {}",
                        asset.browser_download_url
                    );
                    create_game_dir("twelve-knights".to_string()).unwrap();

                    let target = asset.browser_download_url.clone();
                    tokio::spawn(async move {
                        download_game(target).await;
                    });
                } else {
                    error!("Could not find a valid asset.");
                }
            }

            if ui.button("Play").clicked() {
                error!("NOT IMPLEMENTED");
            }
        });
    }
}

fn create_game_dir(game_id: String) -> Result<(), std::io::Error> {
    let mut dir_builder = DirBuilder::new();
    dir_builder.recursive(true);

    let mut path = get_data_dir();
    path.push(game_id);

    dir_builder.create(path)?;

    Ok(())
}

fn get_game_dir(game_id: String) -> PathBuf {
    let mut path = get_data_dir();
    path.push(game_id);
    path
}

async fn download_game(url: Url) -> Result<(), Box<dyn std::error::Error>> {
    let response = reqwest::get(url).await?;
    let mut dest = {
        let mut fname = get_game_dir("twelve-knights".to_string());
        fname.push("linux-latest.zip");
        println!("File will be downloaded to {}", fname.as_path().display());

        File::create(fname)?
    };

    let content = response.bytes().await?;
    copy(&mut content.as_ref(), &mut dest)?;

    Ok(())
}
