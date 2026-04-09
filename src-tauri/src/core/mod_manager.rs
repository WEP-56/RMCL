use crate::core::downloader::{download_files, DownloadTask};
use crate::models::modrinth::{Version, ModpackIndex};
use crate::models::instance::{Instance, LoaderType};
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
    name: &str,
    version: &Version,
) -> Result<Instance, anyhow::Error> {
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

    // 3. Create the Instance based on dependencies
    let mc_version = index.dependencies.get("minecraft")
        .cloned()
        .unwrap_or_else(|| "1.20.1".to_string());
    
    let mut loader = LoaderType::Vanilla;
    if index.dependencies.contains_key("fabric-loader") {
        loader = LoaderType::Fabric;
    } else if index.dependencies.contains_key("forge") {
        loader = LoaderType::Forge;
    }

    let instance = crate::core::instance::create_instance(name.to_string(), mc_version.clone(), loader)?;
    let instance_dir = get_instance_dir(&instance.id);

    // 4. Download all mods defined in the index
    let mut download_tasks = Vec::new();
    for file_info in &index.files {
        if let Some(env) = &file_info.env {
            if env.client == "unsupported" {
                continue;
            }
        }

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

    // 5. Extract overrides
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };

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

    // 6. Cleanup
    fs::remove_dir_all(temp_dir)?;

    Ok(instance)
}