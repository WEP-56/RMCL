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
    let client = Client::new();
    let total = tasks.len();
    let completed = Arc::new(AtomicUsize::new(0));

    let fetches = stream::iter(tasks).map(|task| {
        let client = client.clone();
        let completed_clone = completed.clone();
        let app_clone = app.clone();
        let task_name_clone = task_name.to_string();
        let instance_id_clone = instance_id.to_string();

        tokio::spawn(async move {
            let mut skip = false;
            if check_file_validity(&task.path, &task.sha1) {
                skip = true;
            } else {
                if let Some(parent) = task.path.parent() {
                    let _ = fs::create_dir_all(parent);
                }

                match client.get(&task.url).send().await {
                    Ok(response) => {
                        let status = response.status();
                        if status.is_success() {
                            if let Ok(bytes) = response.bytes().await {
                                if let Err(e) = fs::write(&task.path, bytes) {
                                    return Err(anyhow::anyhow!("Failed to write file {:?}: {}", task.path, e));
                                }
                            }
                        } else {
                            return Err(anyhow::anyhow!("Failed to download {} (Status: {})", task.url, status));
                        }
                    }
                    Err(e) => return Err(anyhow::anyhow!("Request error for {}: {}", task.url, e)),
                }
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
