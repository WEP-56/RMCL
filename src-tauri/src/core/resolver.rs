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
    let response = client.get(url).send().await?;
    let meta: VersionMeta = response.json().await?;

    // Cache it locally
    let json = serde_json::to_string_pretty(&meta)?;
    fs::write(meta_path, json)?;

    Ok(meta)
}
