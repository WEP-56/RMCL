use crate::core::minecraft::{fetch_version_manifest, VersionManifest};
use crate::models::manifest::{Argument, Arguments, Download, Library, VersionMeta};
use reqwest::Client;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use crate::core::paths;

fn manifest_cache_path(version_id: &str) -> PathBuf {
    let mut path = paths::get_manifests_dir();
    path.push(format!("{}.json", version_id));
    path
}

fn legacy_manifest_cache_path(version_id: &str) -> PathBuf {
    let mut path = crate::core::config::get_app_dir();
    path.push(".minecraft");
    path.push("versions");
    path.push(version_id);
    path.push(format!("{}.json", version_id));
    path
}

pub async fn fetch_version_meta(version_id: &str, url: &str) -> Result<VersionMeta, anyhow::Error> {
    let meta_path = manifest_cache_path(version_id);

    // Try to load from local cache first
    if meta_path.exists() {
        let data = fs::read_to_string(&meta_path)?;
        if let Ok(meta) = serde_json::from_str(&data) {
            return Ok(meta);
        }
    }

    let legacy_meta_path = legacy_manifest_cache_path(version_id);
    if legacy_meta_path.exists() {
        let data = fs::read_to_string(&legacy_meta_path)?;
        if let Ok(meta) = serde_json::from_str(&data) {
            let json = serde_json::to_string_pretty(&meta)?;
            fs::write(&meta_path, json)?;
            return Ok(meta);
        }
    }

    // Download if not exists or invalid
    let client = Client::new();
    
    // Apply BMCLAPI mirror fallback if the URL is from mojang
    let final_url = url.to_string();
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

pub async fn fetch_resolved_version_meta(
    version_id: &str,
    url: &str,
) -> Result<VersionMeta, anyhow::Error> {
    let manifest = fetch_version_manifest().await?;
    fetch_resolved_version_meta_with_manifest(version_id, url, &manifest).await
}

pub async fn fetch_resolved_version_meta_with_manifest(
    version_id: &str,
    url: &str,
    manifest: &VersionManifest,
) -> Result<VersionMeta, anyhow::Error> {
    let mut chain = Vec::new();
    let mut current_version_id = version_id.to_string();
    let mut current_url = url.to_string();
    let mut visited = HashSet::new();

    loop {
        if !visited.insert(current_version_id.clone()) {
            return Err(anyhow::anyhow!(
                "Detected cyclic inheritsFrom chain while resolving {}",
                version_id
            ));
        }

        let meta = fetch_version_meta(&current_version_id, &current_url).await?;
        let parent_id = meta.inherits_from.clone();
        chain.push(meta);

        let Some(parent_id) = parent_id else {
            break;
        };

        let parent_url = manifest
            .versions
            .iter()
            .find(|version| version.id == parent_id)
            .map(|version| version.url.clone())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Parent version {} referenced by {} was not found in the version manifest",
                    parent_id,
                    current_version_id
                )
            })?;

        current_version_id = parent_id;
        current_url = parent_url;
    }

    let mut iter = chain.into_iter().rev();
    let mut resolved = iter
        .next()
        .ok_or_else(|| anyhow::anyhow!("No version metadata loaded for {}", version_id))?;

    for child in iter {
        resolved = merge_version_meta(resolved, child);
    }

    resolved.inherits_from = None;
    Ok(resolved)
}

pub fn merge_version_meta(parent: VersionMeta, child: VersionMeta) -> VersionMeta {
    VersionMeta {
        id: child.id,
        version_type: prefer_child_string(parent.version_type, child.version_type),
        main_class: prefer_child_string(parent.main_class, child.main_class),
        inherits_from: None,
        minecraft_arguments: child.minecraft_arguments.or(parent.minecraft_arguments),
        arguments: merge_arguments(parent.arguments, child.arguments),
        asset_index: child.asset_index.or(parent.asset_index),
        assets: child.assets.or(parent.assets),
        downloads: merge_downloads(parent.downloads, child.downloads),
        libraries: merge_libraries(parent.libraries, child.libraries),
        java_version: child.java_version.or(parent.java_version),
        logging: child.logging.or(parent.logging),
    }
}

fn prefer_child_string(parent: String, child: String) -> String {
    if child.trim().is_empty() {
        parent
    } else {
        child
    }
}

fn merge_arguments(parent: Option<Arguments>, child: Option<Arguments>) -> Option<Arguments> {
    match (parent, child) {
        (Some(parent), Some(child)) => Some(Arguments {
            game: merge_argument_lists(parent.game, child.game),
            jvm: merge_argument_lists(parent.jvm, child.jvm),
        }),
        (Some(parent), None) => Some(parent),
        (None, Some(child)) => Some(child),
        (None, None) => None,
    }
}

fn merge_argument_lists(
    parent: Option<Vec<Argument>>,
    child: Option<Vec<Argument>>,
) -> Option<Vec<Argument>> {
    match (parent, child) {
        (Some(mut parent), Some(child)) => {
            parent.extend(child);
            Some(parent)
        }
        (Some(parent), None) => Some(parent),
        (None, Some(child)) => Some(child),
        (None, None) => None,
    }
}

fn merge_downloads(
    parent: Option<HashMap<String, Download>>,
    child: Option<HashMap<String, Download>>,
) -> Option<HashMap<String, Download>> {
    match (parent, child) {
        (Some(mut parent), Some(child)) => {
            parent.extend(child);
            Some(parent)
        }
        (Some(parent), None) => Some(parent),
        (None, Some(child)) => Some(child),
        (None, None) => None,
    }
}

fn merge_libraries(parent: Vec<Library>, child: Vec<Library>) -> Vec<Library> {
    let child_keys: HashSet<String> = child
        .iter()
        .map(|library| library_override_key(&library.name))
        .collect();

    let mut merged: Vec<Library> = parent
        .into_iter()
        .filter(|library| !child_keys.contains(&library_override_key(&library.name)))
        .collect();
    merged.extend(child);
    merged
}

fn library_override_key(name: &str) -> String {
    let parts: Vec<&str> = name.split(':').collect();
    if parts.len() < 2 {
        return name.to_string();
    }

    if parts.len() >= 4 {
        let classifier = parts[3].split('@').next().unwrap_or(parts[3]);
        return format!("{}:{}:{}", parts[0], parts[1], classifier);
    }

    format!("{}:{}", parts[0], parts[1])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::manifest::{JavaVersion, LoggingConfig, Rule};

    fn version_meta(id: &str) -> VersionMeta {
        VersionMeta {
            id: id.to_string(),
            version_type: "release".to_string(),
            main_class: "net.minecraft.client.main.Main".to_string(),
            inherits_from: None,
            minecraft_arguments: None,
            arguments: None,
            asset_index: None,
            assets: None,
            downloads: None,
            libraries: vec![],
            java_version: None,
            logging: None,
        }
    }

    fn library(name: &str) -> Library {
        Library {
            name: name.to_string(),
            downloads: None,
            extract: None,
            natives: None,
            rules: Some(vec![Rule {
                action: "allow".to_string(),
                os: None,
                features: None,
            }]),
        }
    }

    #[test]
    fn merge_version_meta_inherits_parent_defaults() {
        let mut parent = version_meta("1.20.1");
        parent.assets = Some("1.20".to_string());
        parent.java_version = Some(JavaVersion {
            component: "jre-legacy".to_string(),
            major_version: 17,
        });
        parent.logging = Some(LoggingConfig { client: None });

        let mut child = version_meta("fabric-loader");
        child.version_type = "".to_string();
        child.main_class = "".to_string();

        let merged = merge_version_meta(parent, child);

        assert_eq!(merged.id, "fabric-loader");
        assert_eq!(merged.version_type, "release");
        assert_eq!(merged.main_class, "net.minecraft.client.main.Main");
        assert_eq!(merged.assets.as_deref(), Some("1.20"));
        assert_eq!(merged.java_version.as_ref().map(|java| java.major_version), Some(17));
        assert!(merged.logging.is_some());
    }

    #[test]
    fn merge_version_meta_appends_child_arguments() {
        let mut parent = version_meta("parent");
        parent.arguments = Some(Arguments {
            game: Some(vec![Argument::String("--demo".to_string())]),
            jvm: Some(vec![Argument::String("-Xms1G".to_string())]),
        });

        let mut child = version_meta("child");
        child.arguments = Some(Arguments {
            game: Some(vec![Argument::String("--username".to_string())]),
            jvm: Some(vec![Argument::String("-Xmx2G".to_string())]),
        });

        let merged = merge_version_meta(parent, child);
        let arguments = merged.arguments.expect("arguments should exist");

        assert_eq!(arguments.game.expect("game args").len(), 2);
        assert_eq!(arguments.jvm.expect("jvm args").len(), 2);
    }

    #[test]
    fn merge_version_meta_lets_child_override_library_family() {
        let mut parent = version_meta("parent");
        parent.libraries = vec![
            library("org.example:demo:1.0.0"),
            library("org.example:keep:1.0.0"),
        ];

        let mut child = version_meta("child");
        child.libraries = vec![
            library("org.example:demo:2.0.0"),
            library("org.example:new:1.0.0"),
        ];

        let merged = merge_version_meta(parent, child);
        let library_names: Vec<&str> = merged.libraries.iter().map(|library| library.name.as_str()).collect();

        assert_eq!(
            library_names,
            vec![
                "org.example:keep:1.0.0",
                "org.example:demo:2.0.0",
                "org.example:new:1.0.0"
            ]
        );
    }
}
