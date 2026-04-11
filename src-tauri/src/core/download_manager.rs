use crate::core::downloader::{download_files, DownloadTask};
use crate::core::paths;
use crate::core::rules::evaluate_rules;
use crate::models::manifest::VersionMeta;
use std::collections::HashMap;
use std::path::PathBuf;

pub async fn download_libraries(
    meta: &VersionMeta,
    app: Option<tauri::AppHandle>,
    instance_id: &str,
) -> Result<(), anyhow::Error> {
    let mut tasks = Vec::new();
    let lib_dir = paths::get_libraries_dir();
    let active_features = HashMap::new();

    for lib in &meta.libraries {
        if let Some(rules) = &lib.rules {
            if !evaluate_rules(rules, &active_features) {
                continue;
            }
        }

        // 1. Download regular artifact
        if let Some(downloads) = &lib.downloads {
            if let Some(artifact) = &downloads.artifact {
                let path = lib_dir.join(&artifact.path);
                tasks.push(DownloadTask {
                    url: artifact.url.clone(),
                    path,
                    sha1: Some(artifact.sha1.clone()),
                    size: Some(artifact.size),
                });
            }

            // 2. Download natives
            if let Some(natives) = &lib.natives {
                #[cfg(target_os = "windows")]
                let os_key = "windows";
                #[cfg(target_os = "macos")]
                let os_key = "osx";
                #[cfg(target_os = "linux")]
                let os_key = "linux";

                if let Some(native_classifier) = natives.get(os_key) {
                    // Replace "${arch}" with actual architecture
                    #[cfg(target_arch = "x86_64")]
                    let arch_str = "64";
                    #[cfg(target_arch = "x86")]
                    let arch_str = "32";
                    #[cfg(target_arch = "aarch64")]
                    let arch_str = "arm64";

                    let actual_classifier = native_classifier.replace("${arch}", arch_str);

                    if let Some(classifiers) = &downloads.classifiers {
                        if let Some(native_artifact) = classifiers.get(&actual_classifier) {
                            let path = lib_dir.join(&native_artifact.path);
                            tasks.push(DownloadTask {
                                url: native_artifact.url.clone(),
                                path,
                                sha1: Some(native_artifact.sha1.clone()),
                                size: Some(native_artifact.size),
                            });
                        }
                    }
                }
            }
        }
    }

    download_files(tasks, 16, app, instance_id, "下载核心库").await?;
    Ok(())
}

pub async fn download_client_jar(
    meta: &VersionMeta,
    app: Option<tauri::AppHandle>,
    instance_id: &str,
) -> Result<(), anyhow::Error> {
    if let Some(downloads) = &meta.downloads {
        if let Some(client) = downloads.get("client") {
            let mut path = paths::get_versions_dir();
            path.push(&meta.id);
            std::fs::create_dir_all(&path)?;
            path.push(format!("{}.jar", meta.id));

            let task = DownloadTask {
                url: client.url.clone(),
                path,
                sha1: Some(client.sha1.clone()),
                size: Some(client.size),
            };

            download_files(vec![task], 1, app, instance_id, "下载游戏核心").await?;
        }
    }
    Ok(())
}

pub async fn download_logging_config(
    meta: &VersionMeta,
    app: Option<tauri::AppHandle>,
    instance_id: &str,
) -> Result<Option<PathBuf>, anyhow::Error> {
    let Some(logging_client) = meta
        .logging
        .as_ref()
        .and_then(|logging| logging.client.as_ref())
    else {
        return Ok(None);
    };

    let Some(logging_file) = logging_client.file.as_ref() else {
        return Ok(None);
    };

    let mut path = paths::get_versions_dir();
    path.push(&meta.id);
    std::fs::create_dir_all(&path)?;
    path.push(&logging_file.id);

    let task = DownloadTask {
        url: logging_file.url.clone(),
        path: path.clone(),
        sha1: Some(logging_file.sha1.clone()),
        size: Some(logging_file.size),
    };

    download_files(vec![task], 1, app, instance_id, "下载日志配置").await?;
    Ok(Some(path))
}
