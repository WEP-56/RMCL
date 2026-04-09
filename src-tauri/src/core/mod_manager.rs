use crate::core::downloader::{download_files, DownloadTask};
use crate::models::modrinth::{Version, ModpackIndex};
use std::fs;
use std::path::PathBuf;
use std::io::{Read, Write};
use zip::ZipArchive;

pub fn get_instance_mods_dir(instance_id: &str) -> PathBuf {
    let mut path = crate::core::instance::get_instances_dir();
    path.push(instance_id);
    path.push("mods");
    if !path.exists() {
        fs::create_dir_all(&path).unwrap();
    }
    path
}

pub fn get_instance_dir(instance_id: &str) -> PathBuf {
    let mut path = crate::core::instance::get_instances_dir();
    path.push(instance_id);
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

pub async fn install_modpack(
    instance_id: &str,
    version: &Version,
) -> Result<(), anyhow::Error> {
    let instance_dir = get_instance_dir(instance_id);
    let target_file = version.files.iter().find(|f| f.primary).unwrap_or_else(|| &version.files[0]);
    
    // 1. Download the .mrpack file to a temporary location
    let temp_dir = std::env::temp_dir().join(format!("rustmc_mrpack_{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&temp_dir)?;
    let mrpack_path = temp_dir.join(&target_file.filename);

    let task = DownloadTask {
        url: target_file.url.clone(),
        path: mrpack_path.clone(),
        sha1: Some(target_file.hashes.sha1.clone()),
        size: Some(target_file.size as u64),
    };

    download_files(vec![task], 1).await?;

    // 2. Unzip and parse the .mrpack
    let file = fs::File::open(&mrpack_path)?;
    let mut archive = ZipArchive::new(file)?;

    // Read modrinth.index.json
    let mut index_content = String::new();
    {
        let mut index_file = archive.by_name("modrinth.index.json")
            .map_err(|_| anyhow::anyhow!("Invalid .mrpack: missing modrinth.index.json"))?;
        index_file.read_to_string(&mut index_content)?;
    }

    let index: ModpackIndex = serde_json::from_str(&index_content)?;

    // 3. Download all mods defined in the index
    let mut download_tasks = Vec::new();
    for file_info in &index.files {
        // Skip server-only mods
        if let Some(env) = &file_info.env {
            if env.client == "unsupported" {
                continue;
            }
        }

        // Construct the correct target path inside the instance (e.g., mods/sodium.jar)
        let target_path = instance_dir.join(&file_info.path);
        
        if let Some(url) = file_info.downloads.first() {
            download_tasks.push(DownloadTask {
                url: url.clone(),
                path: target_path,
                sha1: Some(file_info.hashes.sha1.clone()),
                size: Some(file_info.file_size as u64),
            });
        }
    }

    if !download_tasks.is_empty() {
        download_files(download_tasks, 16).await?;
    }

    // 4. Extract overrides (configs, resourcepacks, scripts, etc.)
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };

        // Modrinth pack overrides are usually inside an "overrides/" folder
        // We need to strip "overrides/" and extract the rest to instance root
        if let Ok(stripped) = outpath.strip_prefix("overrides/") {
            let target_path = instance_dir.join(stripped);

            if file.is_dir() {
                fs::create_dir_all(&target_path)?;
            } else {
                if let Some(parent) = target_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                let mut outfile = fs::File::create(&target_path)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
        }
    }

    // 5. Cleanup
    fs::remove_dir_all(temp_dir)?;

    Ok(())
}