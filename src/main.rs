mod file_management;

mod prelude {
    pub use crate::file_management::*;
    pub use eframe::egui;
    pub use log::{error, info, trace, warn};
    pub use octocrab::{self, models::repos::Release};
    pub use pretty_env_logger;
    pub use reqwest::Url;
    pub use std::fs::DirBuilder;
    pub use std::fs::File;
    pub use std::io::copy;
    pub use std::path::PathBuf;
    pub use std::process::Command;
}

use prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();
    create_data_dir();
    let options = eframe::NativeOptions::default();

    let mut app = LauncherApp::new().await?;

    eframe::run_native("Arcana Launcher", options, Box::new(|_cc| Box::new(app)));
}

struct LauncherApp {
    latest_twelve_knights: Release,
    download_status: Option<DownloadStatus>,
}

#[derive(PartialEq)]
enum DownloadStatus {
    Downloading,
    Extracting,
}

impl LauncherApp {
    async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let release = Self::get_latest_twelve_knights().await?;
        Ok(Self {
            latest_twelve_knights: release,
            download_status: None,
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

            let download_button = egui::Button::new("Download");
            if ui
                .add_enabled(self.download_status == None, download_button)
                .clicked()
            {
                trace!("Finding correct download link.");
                if let Some(asset) = self
                    .latest_twelve_knights
                    .assets
                    .iter()
                    .find(|asset| asset.name == get_download_name_for_os())
                {
                    trace!(
                        "Found asset with download link: {}",
                        asset.browser_download_url
                    );
                    create_game_dir("twelve-knights-vigil".to_string()).unwrap();

                    let target = asset.browser_download_url.clone();
                    tokio::spawn(async move {
                        download_game(target).await;
                    });
                } else {
                    error!("Could not find a valid asset.");
                }
            }

            let play_button = egui::Button::new("Play");
            let file_exists =
                std::path::Path::new(&get_os_executable("twelve-knights-vigil".to_string()))
                    .exists();
            if ui
                .add_enabled(file_exists && self.download_status == None, play_button)
                .clicked()
            {
                let working_dir = get_os_working_dir("twelve-knights-vigil".to_string());
                let executable = get_os_executable("twelve-knights-vigil".to_string());
                Command::new(executable)
                    .current_dir(working_dir.clone())
                    .env("CARGO_MANIFEST_DIR", working_dir.clone())
                    .spawn()
                    .expect("Failed to launch game");
            }
        });
    }
}

async fn download_game(url: Url) -> Result<(), Box<dyn std::error::Error>> {
    let response = reqwest::get(url).await?;
    let mut fname = get_game_dir("twelve-knights-vigil".to_string());
    fname.push(get_download_name_for_os());

    println!("File will be downloaded to {}", fname.as_path().display());
    let mut zip_file = File::create(fname.clone())?;

    let content = response.bytes().await?;
    copy(&mut content.as_ref(), &mut zip_file)?;

    info!("Finished Download!");
    info!("Beginning Extraction...");

    let file = std::fs::File::open(fname.clone())?;
    let mut archive = zip::ZipArchive::new(&file).unwrap();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let outpath = match file.enclosed_name() {
            Some(path) => get_game_dir("twelve-knights-vigil".to_string()).join(path.to_owned()),
            None => continue,
        };

        if (*file.name()).ends_with('/') {
            info!("File {} extracted to \"{}\"", i, outpath.display());
            std::fs::create_dir_all(&outpath).unwrap();
        } else {
            info!(
                "File {} extracted to \"{}\" ({} bytes)",
                i,
                outpath.display(),
                file.size()
            );
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(&p).unwrap();
                }
            }

            let mut outfile = std::fs::File::create(&outpath).unwrap();
            copy(&mut file, &mut outfile).unwrap();
        }

        // Get and Set Permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = file.unix_mode() {
                std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode)).unwrap();
            }
        }
    }

    Ok(())
}
