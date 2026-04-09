use crate::core::downloader::{download_files, DownloadTask};
use crate::models::modrinth::Version;
use std::fs;
use std::path::PathBuf;

pub fn get_instance_mods_dir(instance_id: &str) -> PathBuf {
    let mut path = crate::core::instance::get_instances_dir();
    path.push(instance_id);
    path.push("mods");
    if !path.exists() {
        fs::create_dir_all(&path).unwrap();
    }
    path
}

pub async fn install_mod_version(
    instance_id: &str,
    version: &Version,
) -> Result<(), anyhow::Error> {
    let mods_dir = get_instance_mods_dir(instance_id);
    let mut tasks = Vec::new();

    // Find the primary file, or just use the first one
    let target_file = version.files.iter().find(|f| f.primary).unwrap_or_else(|| &version.files[0]);

    let file_path = mods_dir.join(&target_file.filename);
    
    tasks.push(DownloadTask {
        url: target_file.url.clone(),
        path: file_path,
        sha1: Some(target_file.hashes.sha1.clone()),
        size: Some(target_file.size as u64),
    });

    download_files(tasks, 1).await?;
    
    Ok(())
}
