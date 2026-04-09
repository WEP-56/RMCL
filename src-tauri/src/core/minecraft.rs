use serde::{Deserialize, Serialize};

const VERSION_MANIFEST_URL: &str =
    "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
const BMCLAPI_MANIFEST_URL: &str =
    "https://bmclapi2.bangbang93.com/mc/game/version_manifest_v2.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionManifest {
    pub latest: LatestVersions,
    pub versions: Vec<VersionInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LatestVersions {
    pub release: String,
    pub snapshot: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionInfo {
    pub id: String,
    #[serde(rename = "type")]
    pub version_type: String,
    pub url: String,
    pub time: String,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
}

pub async fn fetch_version_manifest() -> Result<VersionManifest, anyhow::Error> {
    let client = reqwest::Client::new();
    
    // 尝试使用官方源
    let response = match client.get(VERSION_MANIFEST_URL).send().await {
        Ok(res) if res.status().is_success() => res,
        // 失败则自动回退到 BMCLAPI
        _ => client.get(BMCLAPI_MANIFEST_URL).send().await?
    };
    
    let manifest: VersionManifest = response.json().await?;
    Ok(manifest)
}
