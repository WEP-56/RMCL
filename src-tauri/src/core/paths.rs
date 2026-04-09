use std::path::PathBuf;
use crate::core::config::load_settings;

pub fn get_minecraft_dir() -> PathBuf {
    if let Ok(settings) = load_settings() {
        if let Some(custom_dir) = settings.game_directory {
            if !custom_dir.trim().is_empty() {
                return PathBuf::from(custom_dir);
            }
        }
    }

    let mut path = crate::core::config::get_app_dir();
    path.push(".minecraft");
    path
}

pub fn get_versions_dir() -> PathBuf {
    let mut path = get_minecraft_dir();
    path.push("versions");
    path
}

pub fn get_libraries_dir() -> PathBuf {
    let mut path = get_minecraft_dir();
    path.push("libraries");
    path
}

pub fn get_assets_dir() -> PathBuf {
    let mut path = get_minecraft_dir();
    path.push("assets");
    path
}
