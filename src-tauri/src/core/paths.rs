use std::path::PathBuf;

pub fn get_minecraft_dir() -> PathBuf {
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
