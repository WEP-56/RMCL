use crate::core::downloader::{download_files, DownloadTask};
use crate::core::paths;
use crate::models::manifest::AssetObjects;
use std::fs;

pub async fn download_assets(asset_index_url: &str, asset_index_id: &str, app: Option<tauri::AppHandle>, instance_id: &str) -> Result<(), anyhow::Error> {
    let assets_dir = paths::get_assets_dir();
    
    // Download and parse the asset index json
    let indexes_dir = assets_dir.join("indexes");
    if !indexes_dir.exists() {
        fs::create_dir_all(&indexes_dir)?;
    }
    
    let index_file = indexes_dir.join(format!("{}.json", asset_index_id));
    
    if !index_file.exists() {
        let task = DownloadTask {
            url: asset_index_url.to_string(),
            path: index_file.clone(),
            sha1: None,
            size: None,
        };
        download_files(vec![task], 1, app.clone(), instance_id, "下载资源索引").await?;
    }
    
    let index_json = fs::read_to_string(&index_file)?;

    let asset_objects: AssetObjects = serde_json::from_str(&index_json)?;
    let objects_dir = assets_dir.join("objects");
    let mut tasks = Vec::new();

    for (_, object) in asset_objects.objects {
        let hash = &object.hash;
        let prefix = &hash[0..2];
        let path = objects_dir.join(prefix).join(hash);
        
        let url = format!("https://resources.download.minecraft.net/{}/{}", prefix, hash);
        
        // Ensure path directory exists for the download task
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        tasks.push(DownloadTask {
            url,
            path,
            sha1: Some(hash.clone()),
            size: Some(object.size),
        });
    }

    // Download assets with higher concurrency since they are many small files
    if !tasks.is_empty() {
        download_files(tasks, 32, app, instance_id, "下载游戏资源(Assets)").await?;
    }
    
    Ok(())
}
