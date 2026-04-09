use crate::core::downloader::{download_files, DownloadTask};
use crate::core::paths;
use crate::models::manifest::{Rule, VersionMeta};

// 判断操作系统和架构，目前仅支持 Windows 简单判断，后续可扩展
fn match_rule(rules: &Option<Vec<Rule>>) -> bool {
    if let Some(rules) = rules {
        let mut allowed = false;
        for rule in rules {
            if rule.action == "allow" {
                if let Some(os) = &rule.os {
                    if os.name.as_deref() == Some("windows") {
                        allowed = true;
                    }
                } else {
                    allowed = true; // No OS specified, globally allowed
                }
            } else if rule.action == "disallow" {
                if let Some(os) = &rule.os {
                    if os.name.as_deref() == Some("windows") {
                        allowed = false;
                    }
                }
            }
        }
        return allowed;
    }
    true // If no rules, implicitly allowed
}

pub async fn download_libraries(meta: &VersionMeta, app: Option<tauri::AppHandle>, instance_id: &str) -> Result<(), anyhow::Error> {
    let mut tasks = Vec::new();
    let lib_dir = paths::get_libraries_dir();

    for lib in &meta.libraries {
        if !match_rule(&lib.rules) {
            continue;
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
                if let Some(native_classifier) = natives.get("windows") { // TODO: dynamic OS
                    // Replace "${arch}" with actual architecture (e.g. "64") if present in the string
                    let actual_classifier = native_classifier.replace("${arch}", "64");
                    
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

pub async fn download_client_jar(meta: &VersionMeta, app: Option<tauri::AppHandle>, instance_id: &str) -> Result<(), anyhow::Error> {
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
