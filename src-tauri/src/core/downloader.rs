use reqwest::Client;
use sha1::{Digest, Sha1};
use std::fs;
use std::path::PathBuf;
use futures::stream::{self, StreamExt};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tauri::Emitter;

#[derive(Debug, Clone, serde::Serialize)]
pub struct ProgressPayload {
    pub instance_id: String,
    pub task: String,
    pub progress: f64,
    pub text: String,
}
#[derive(Debug, Clone)]
pub struct DownloadTask {
    pub url: String,
    pub path: PathBuf,
    pub sha1: Option<String>,
    pub size: Option<u64>,
}

fn check_file_validity(path: &PathBuf, expected_sha1: &Option<String>) -> bool {
    if !path.exists() {
        return false;
    }

    if let Some(sha1) = expected_sha1 {
        if let Ok(data) = fs::read(path) {
            let mut hasher = Sha1::new();
            hasher.update(&data);
            let result = format!("{:x}", hasher.finalize());
            return result == *sha1;
        }
        return false;
    }

    true // If no sha1 provided, assume valid if exists
}

pub async fn download_files(
    tasks: Vec<DownloadTask>,
    concurrency: usize,
    app: Option<tauri::AppHandle>,
    instance_id: &str,
    task_name: &str,
) -> Result<(), anyhow::Error> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap_or_default();
        
    let total = tasks.len();
    let completed = Arc::new(AtomicUsize::new(0));
    
    let settings = crate::core::config::load_settings().unwrap_or_default();
    let use_bmclapi = settings.download_source.as_deref() == Some("BMCLAPI");

    let fetches = stream::iter(tasks).map(|task| {
        let client = client.clone();
        let completed_clone = completed.clone();
        let app_clone = app.clone();
        let task_name_clone = task_name.to_string();
        let instance_id_clone = instance_id.to_string();

        tokio::spawn(async move {
            // First check if the URL ends with a colon, which might be a bug in upstream or parsing
            let mut final_url = task.url.clone();
            if final_url.ends_with(':') {
                final_url.pop();
            }
            
            // Apply BMCLAPI Mirror if enabled
            if use_bmclapi {
                if final_url.starts_with("https://resources.download.minecraft.net") {
                    final_url = final_url.replace("https://resources.download.minecraft.net", "https://bmclapi2.bangbang93.com/assets");
                } else if final_url.starts_with("http://resources.download.minecraft.net") {
                    final_url = final_url.replace("http://resources.download.minecraft.net", "https://bmclapi2.bangbang93.com/assets");
                } else if final_url.starts_with("https://libraries.minecraft.net") {
                    final_url = final_url.replace("https://libraries.minecraft.net", "https://bmclapi2.bangbang93.com/maven");
                } else if final_url.starts_with("https://launcher.mojang.com") {
                    final_url = final_url.replace("https://launcher.mojang.com", "https://bmclapi2.bangbang93.com");
                } else if final_url.starts_with("https://piston-meta.mojang.com") {
                    final_url = final_url.replace("https://piston-meta.mojang.com", "https://bmclapi2.bangbang93.com");
                } else if final_url.starts_with("https://meta.fabricmc.net") {
                    final_url = final_url.replace("https://meta.fabricmc.net", "https://bmclapi2.bangbang93.com/fabric-meta");
                } else if final_url.starts_with("https://maven.fabricmc.net") {
                    final_url = final_url.replace("https://maven.fabricmc.net", "https://bmclapi2.bangbang93.com/maven");
                }
            }

            if check_file_validity(&task.path, &task.sha1) {
                // Skip if valid file exists
                let current = completed_clone.fetch_add(1, Ordering::Relaxed) + 1;
                if let Some(a) = &app_clone {
                    let _ = a.emit("mc-progress", ProgressPayload {
                        instance_id: instance_id_clone,
                        task: task_name_clone,
                        progress: current as f64 / total as f64,
                        text: format!("{} / {}", current, total),
                    });
                }
                return Ok(());
            }

            if let Some(parent) = task.path.parent() {
                let _ = fs::create_dir_all(parent);
            }

            let mut attempts = 0;
            let max_attempts = 3;
            let mut last_err = String::new();

            while attempts < max_attempts {
                // If BMCLAPI fails, fallback to official Mojang source on the last attempt
                if attempts == max_attempts - 1 && use_bmclapi {
                    if final_url.contains("bmclapi2.bangbang93.com") {
                        final_url = task.url.clone();
                        if final_url.ends_with(':') {
                            final_url.pop();
                        }
                    }
                }

                match client.get(&final_url).send().await {
                    Ok(response) => {
                        let status = response.status();
                        if status.is_success() {
                            if let Ok(bytes) = response.bytes().await {
                                if let Err(e) = fs::write(&task.path, bytes) {
                                    return Err(anyhow::anyhow!("Failed to write file {:?}: {}", task.path, e));
                                }
                                // Check sha1 if needed after download
                                if !check_file_validity(&task.path, &task.sha1) {
                                    attempts += 1;
                                    last_err = "SHA1 mismatch".to_string();
                                    continue;
                                }
                                break; // Success
                            } else {
                                attempts += 1;
                                last_err = "Failed to read bytes".to_string();
                            }
                        } else {
                            attempts += 1;
                            last_err = format!("Status: {}", status);
                        }
                    }
                    Err(e) => {
                        attempts += 1;
                        last_err = format!("Request error: {}", e);
                    }
                }
            }

            if attempts == max_attempts {
                return Err(anyhow::anyhow!("Failed to download {} after {} attempts. Last error: {}", final_url, max_attempts, last_err));
            }

            // Report progress
            let current = completed_clone.fetch_add(1, Ordering::Relaxed) + 1;
            if let Some(a) = &app_clone {
                let _ = a.emit("mc-progress", ProgressPayload {
                    instance_id: instance_id_clone,
                    task: task_name_clone,
                    progress: current as f64 / total as f64,
                    text: format!("{} / {}", current, total),
                });
            }

            Ok(())
        })
    })
    .buffer_unordered(concurrency);

    // Collect all results and check for errors
    let results: Vec<Result<Result<(), anyhow::Error>, tokio::task::JoinError>> = fetches.collect().await;

    for res in results {
        match res {
            Ok(Ok(_)) => continue,
            Ok(Err(e)) => return Err(e), // Internal download error
            Err(e) => return Err(anyhow::anyhow!("Task panicked: {}", e)),
        }
    }

    Ok(())
}
