use crate::core::config::load_settings;
use std::fs;
use std::path::PathBuf;

fn ensure_dir(path: PathBuf) -> PathBuf {
    if !path.exists() {
        fs::create_dir_all(&path).unwrap();
    }
    path
}

pub fn get_runtime_dir() -> PathBuf {
    if let Ok(settings) = load_settings() {
        if let Some(custom_dir) = settings.game_directory {
            if !custom_dir.trim().is_empty() {
                return ensure_dir(PathBuf::from(custom_dir));
            }
        }
    }

    let mut path = crate::core::config::get_app_dir();
    path.push("runtime");
    ensure_dir(path)
}

pub fn get_versions_dir() -> PathBuf {
    let mut path = get_runtime_dir();
    path.push("versions");
    ensure_dir(path)
}

pub fn get_libraries_dir() -> PathBuf {
    let mut path = get_runtime_dir();
    path.push("libraries");
    ensure_dir(path)
}

pub fn get_assets_dir() -> PathBuf {
    let mut path = get_runtime_dir();
    path.push("assets");
    ensure_dir(path)
}

pub fn get_java_dir() -> PathBuf {
    let mut path = get_runtime_dir();
    path.push("java");
    ensure_dir(path)
}

pub fn get_manifests_dir() -> PathBuf {
    let mut path = get_runtime_dir();
    path.push("manifests");
    ensure_dir(path)
}
