use crate::core::downloader::{download_files, DownloadTask};
use crate::models::modrinth::{Version, ModpackIndex};
use crate::models::instance::{Instance, LoaderType};
use std::fs::{self, File};
use std::io::{Read, Write};
use zip::ZipArchive;
use zip::write::FileOptions;
use serde::{Serialize, Deserialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LocalMod {
    pub name: String,
    pub path: String,
    pub enabled: bool,
}

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
    app: Option<tauri::AppHandle>,
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

    download_files(tasks, 1, app, instance_id, "安装模组").await?;
    
    Ok(())
}

pub async fn install_modpack(
    name: &str,
    version: &Version,
    app: Option<tauri::AppHandle>,
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

    let temp_id = uuid::Uuid::new_v4().to_string();
    download_files(vec![task], 1, app.clone(), &temp_id, "下载整合包本体").await?;

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
        download_files(download_tasks, 16, app, &instance.id, "下载整合包模组").await?;
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

pub fn get_local_mods(instance_id: &str) -> Result<Vec<LocalMod>, anyhow::Error> {
    let mods_dir = get_instance_mods_dir(instance_id);
    let mut mods = Vec::new();

    if !mods_dir.exists() {
        return Ok(mods);
    }

    for entry in fs::read_dir(mods_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let file_name = entry.file_name().to_string_lossy().to_string();
            if file_name.ends_with(".jar") || file_name.ends_with(".jar.disabled") {
                let enabled = file_name.ends_with(".jar");
                mods.push(LocalMod {
                    name: file_name,
                    path: path.to_string_lossy().to_string(),
                    enabled,
                });
            }
        }
    }

    Ok(mods)
}

pub fn toggle_mod(instance_id: &str, mod_name: &str, enabled: bool) -> Result<(), anyhow::Error> {
    let mods_dir = get_instance_mods_dir(instance_id);
    let current_path = mods_dir.join(mod_name);
    
    if !current_path.exists() {
        return Err(anyhow::anyhow!("Mod file not found"));
    }

    let new_name = if enabled {
        if mod_name.ends_with(".disabled") {
            mod_name.trim_end_matches(".disabled").to_string()
        } else {
            mod_name.to_string()
        }
    } else {
        if !mod_name.ends_with(".disabled") {
            format!("{}.disabled", mod_name)
        } else {
            mod_name.to_string()
        }
    };

    let new_path = mods_dir.join(&new_name);
    fs::rename(current_path, new_path)?;

    Ok(())
}

pub fn delete_mod(instance_id: &str, mod_name: &str) -> Result<(), anyhow::Error> {
    let mods_dir = get_instance_mods_dir(instance_id);
    let path = mods_dir.join(mod_name);
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

pub fn open_instance_folder(instance_id: &str) -> Result<(), anyhow::Error> {
    let dir = get_instance_dir(instance_id);
    if !dir.exists() {
        return Err(anyhow::anyhow!("Instance directory does not exist"));
    }
    
    let path_str = dir.to_string_lossy().to_string();

    #[cfg(target_os = "windows")]
    std::process::Command::new("explorer")
        .arg(&path_str)
        .spawn()?;

    #[cfg(target_os = "macos")]
    std::process::Command::new("open")
        .arg(&path_str)
        .spawn()?;

    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open")
        .arg(&path_str)
        .spawn()?;

    Ok(())
}

pub async fn export_instance_to_modrinth(instance_id: &str, output_path: &str) -> Result<(), anyhow::Error> {
    let instance = crate::core::instance::get_instance_by_id(instance_id)?;
    let instance_dir = get_instance_dir(instance_id);
    let mods_dir = instance_dir.join("mods");

    // We only support exporting basic mrpack without actual mod files for now
    // A real implementation would query Modrinth API for each mod's SHA1/SHA512 to build the index
    // For simplicity in this demo, we'll just pack the mods directly into overrides

    let file = File::create(output_path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options: FileOptions<()> = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    // 1. Create a minimal index.json
    let mut deps = std::collections::HashMap::new();
    deps.insert("minecraft".to_string(), instance.mc_version.clone());
    if let LoaderType::Fabric = instance.loader {
        if let Some(loader_ver) = &instance.loader_version {
            deps.insert("fabric-loader".to_string(), loader_ver.clone());
        }
    }

    let index = serde_json::json!({
        "formatVersion": 1,
        "game": "minecraft",
        "versionId": "1.0.0",
        "name": instance.name,
        "summary": "Exported by RustMC Launcher",
        "dependencies": deps,
        "files": [] // We pack everything into overrides for this simple implementation
    });

    zip.start_file("modrinth.index.json", options.clone())?;
    zip.write_all(serde_json::to_string_pretty(&index)?.as_bytes())?;

    // 2. Pack everything as overrides
    let walk_dir = walkdir::WalkDir::new(&instance_dir);
    for entry in walk_dir.into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            // Don't pack instance.json
            if path.file_name() == Some(std::ffi::OsStr::new("instance.json")) {
                continue;
            }

            let name = path.strip_prefix(&instance_dir).unwrap();
            let name_str = name.to_string_lossy().replace("\\", "/");
            let zip_path = format!("overrides/{}", name_str);
            
            zip.start_file(zip_path, options.clone())?;
            let mut f = File::open(path)?;
            std::io::copy(&mut f, &mut zip)?;
        }
    }

    zip.finish()?;
    Ok(())
}