use crate::models::manifest::VersionMeta;
use reqwest::Client;

const FABRIC_META_URL: &str = "https://meta.fabricmc.net/v2";

pub async fn fetch_fabric_meta(game_version: &str, loader_version: &str) -> Result<VersionMeta, anyhow::Error> {
    let url = format!("{}/versions/loader/{}/{}/profile/json", FABRIC_META_URL, game_version, loader_version);
    
    // We can reuse the existing resolver logic to download and parse it!
    // But we need to save it locally with a specific ID format.
    let version_id = format!("fabric-loader-{}-{}", loader_version, game_version);
    
    // Use the existing resolver function
    crate::core::resolver::fetch_version_meta(&version_id, &url).await
}

// Fetch the latest loader version for a given MC version
pub async fn fetch_latest_fabric_loader(game_version: &str) -> Result<String, anyhow::Error> {
    let url = format!("{}/versions/loader/{}", FABRIC_META_URL, game_version);
    let client = Client::new();
    
    let response = client.get(&url).send().await?;
    let loaders: serde_json::Value = response.json().await?;
    
    if let Some(loaders_array) = loaders.as_array() {
        if !loaders_array.is_empty() {
            if let Some(loader_obj) = loaders_array[0].get("loader") {
                if let Some(version) = loader_obj.get("version") {
                    return Ok(version.as_str().unwrap_or("").to_string());
                }
            }
        }
    }
    
    Err(anyhow::anyhow!("No fabric loader found for version {}", game_version))
}
