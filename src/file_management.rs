use crate::prelude::*;
use std::env;

pub fn create_data_dir() {
    let mut dir_builder = DirBuilder::new();
    dir_builder.recursive(true);
    dir_builder.create(get_data_dir().as_path()).unwrap();
}

pub fn get_data_dir() -> PathBuf {
    let mut path = dirs::data_local_dir().unwrap();

    path.push("arcana-launcher");

    path
}

pub fn create_game_dir(game_id: String) -> Result<(), std::io::Error> {
    let mut dir_builder = DirBuilder::new();
    dir_builder.recursive(true);

    let mut path = get_data_dir();
    path.push(game_id);

    dir_builder.create(path)?;

    Ok(())
}

pub fn get_game_dir(game_id: String) -> PathBuf {
    let mut path = get_data_dir();
    path.push(game_id);
    path
}

pub fn get_download_name_for_os() -> String {
    match std::env::consts::OS {
        "linux" => "linux-latest.zip".to_string(),
        "windows" => "windows-latest.zip".to_string(),
        _ => panic!("Invalid Operating System"),
    }
}

pub fn get_os_working_dir(game_id: String) -> PathBuf {
    let mut result = get_game_dir(game_id);
    let dir = match std::env::consts::OS {
        "linux" => "linux-latest",
        "windows" => "windows-latest",
        _ => panic!("Invalid Operating System"),
    };

    result.push(dir.to_string());

    result
}

pub fn get_os_executable(game_id: String) -> PathBuf {
    let mut result = get_os_working_dir(game_id.clone());

    let exe = match std::env::consts::OS {
        "linux" => game_id.clone(),
        "windows" => format!("{}.exe", game_id),
        _ => panic!("INVALID OPERATING SYSTEM"),
    };
    result.push(exe);

    result
}
