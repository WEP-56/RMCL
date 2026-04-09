use crate::models::manifest::VersionMeta;
use reqwest::Client;
use std::fs;
use crate::core::paths;

pub async fn fetch_version_meta(version_id: &str, url: &str) -> Result<VersionMeta, anyhow::Error> {
    let mut version_dir = paths::get_versions_dir();
    version_dir.push(version_id);
    if !version_dir.exists() {
        fs::create_dir_all(&version_dir)?;
    }

    let meta_path = version_dir.join(format!("{}.json", version_id));

    // Try to load from local cache first
    if meta_path.exists() {
        let data = fs::read_to_string(&meta_path)?;
        if let Ok(meta) = serde_json::from_str(&data) {
            return Ok(meta);
        }
    }

    // Download if not exists or invalid
    let client = Client::new();
    
    // Apply BMCLAPI mirror fallback if the URL is from mojang
    let mut final_url = url.to_string();
    if url.contains("piston-meta.mojang.com") || url.contains("launchermeta.mojang.com") {
        let bmcl_url = url.replace("piston-meta.mojang.com", "bmclapi2.bangbang93.com")
                          .replace("launchermeta.mojang.com", "bmclapi2.bangbang93.com");
        
        let response = match client.get(url).send().await {
            Ok(res) if res.status().is_success() => res,
            _ => client.get(&bmcl_url).send().await?
        };
        
        let meta: VersionMeta = response.json().await?;
        let json = serde_json::to_string_pretty(&meta)?;
        fs::write(meta_path, json)?;
        return Ok(meta);
    }

    let response = client.get(&final_url).send().await?;
    let meta: VersionMeta = response.json().await?;

    // Cache it locally
    let json = serde_json::to_string_pretty(&meta)?;
    fs::write(meta_path, json)?;

    Ok(meta)
}
