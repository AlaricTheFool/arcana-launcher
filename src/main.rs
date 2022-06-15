mod file_management;

mod prelude {
    pub use crate::file_management::*;
    pub use eframe::egui;
    pub use futures_util::StreamExt;
    pub use log::{error, info, trace, warn};
    pub use octocrab::{self, models::repos::Release};
    pub use pretty_env_logger;
    pub use reqwest::header::{HeaderValue, CONTENT_LENGTH, RANGE};
    pub use reqwest::StatusCode;
    pub use reqwest::Url;
    pub use std::fs::DirBuilder;
    pub use std::fs::File;
    pub use std::io::copy;
    pub use std::io::Write;
    pub use std::path::PathBuf;
    pub use std::process::Command;
    pub use std::sync::{Arc, Mutex};
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
    download_status: Arc<Mutex<Option<DownloadStatus>>>,
}

#[derive(PartialEq, Copy, Clone)]
enum DownloadStatus {
    Downloading(f32),
    Extracting(f32),
}

type Dstatus = Arc<Mutex<Option<DownloadStatus>>>;

impl LauncherApp {
    async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let release = Self::get_latest_twelve_knights().await?;
        Ok(Self {
            latest_twelve_knights: release,
            download_status: Arc::new(Mutex::new(None)),
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

            const BOTTOM_ROW_HEIGHT: f32 = 64.0;
            const BUTTON_WIDTH: f32 = 128.0;
            let download_status = self.download_status.clone();

            ui.add_space(ui.available_height() - BOTTOM_ROW_HEIGHT);

            ui.with_layout(egui::Layout::right_to_left(), |ui| {
                let file_exists =
                    std::path::Path::new(&get_os_executable("twelve-knights-vigil".to_string()))
                        .exists();
                ui.add_enabled_ui(
                    file_exists && *download_status.lock().unwrap() == None,
                    |ui| {
                        let play_button = egui::Button::new("Play");
                        if ui
                            .add_sized([BUTTON_WIDTH, BOTTOM_ROW_HEIGHT], play_button)
                            .clicked()
                        {
                            let working_dir =
                                get_os_working_dir("twelve-knights-vigil".to_string());
                            let executable = get_os_executable("twelve-knights-vigil".to_string());
                            Command::new(executable)
                                .current_dir(working_dir.clone())
                                .env("CARGO_MANIFEST_DIR", working_dir.clone())
                                .spawn()
                                .expect("Failed to launch game");
                        }
                    },
                );

                let current_status = download_status.lock().unwrap().clone();
                ui.add_enabled_ui(current_status == None, |ui| {
                    let button_text = match current_status {
                        None => "Download",
                        Some(status) => match status {
                            DownloadStatus::Downloading(pct) => "Download in Progress...",
                            DownloadStatus::Extracting(pct) => "Extracting Files...",
                        },
                    };
                    let download_button = egui::Button::new(button_text);
                    if ui
                        .add_sized([BUTTON_WIDTH, BOTTOM_ROW_HEIGHT], download_button)
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

                            let dstatus = self.download_status.clone();
                            let target = asset.browser_download_url.clone();
                            tokio::spawn(async move {
                                download_game(target, dstatus).await.unwrap();
                            });
                        } else {
                            error!("Could not find a valid asset.");
                        }
                    }
                });

                match current_status {
                    None => {
                        ui.label(format!(
                            "Last Update: {}",
                            self.latest_twelve_knights.published_at.unwrap()
                        ));
                    }

                    Some(status) => match status {
                        DownloadStatus::Downloading(pct) | DownloadStatus::Extracting(pct) => {
                            let bar = egui::ProgressBar::new(pct).show_percentage();
                            ui.add(bar);
                        }
                    },
                }
            });
        });
    }
}

async fn download_game(url: Url, status: Dstatus) -> Result<(), Box<dyn std::error::Error>> {
    *status.lock().unwrap() = Some(DownloadStatus::Downloading(0.0));

    let mut fname = get_game_dir("twelve-knights-vigil".to_string());
    fname.push(get_download_name_for_os());
    println!("File will be downloaded to {}", fname.as_path().display());
    let mut zip_file = File::create(fname.clone())?;

    let response = reqwest::get(url).await?;
    let total_size = response
        .content_length()
        .ok_or("Failed to get content length!")?;

    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = 0;

    while let Some(item) = stream.next().await {
        let chunk = item.or(Err(format!("Error while downloading file")))?;
        zip_file.write_all(&chunk)?;
        let new = std::cmp::min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        *status.lock().unwrap() = Some(DownloadStatus::Downloading(
            downloaded as f32 / total_size as f32,
        ));
    }

    /*
    let content = response.bytes().await?;
    copy(&mut content.as_ref(), &mut zip_file)?;
    */

    info!("Finished Download!");
    info!("Beginning Extraction...");
    *status.lock().unwrap() = Some(DownloadStatus::Extracting(0.0));

    let file = std::fs::File::open(fname.clone())?;
    let mut archive = zip::ZipArchive::new(&file).unwrap();

    for i in 0..archive.len() {
        let pct = i as f32 / archive.len() as f32;
        *status.lock().unwrap() = Some(DownloadStatus::Extracting(pct));
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

    *status.lock().unwrap() = None;
    Ok(())
}
