use reqwest::Client;
use sha1::{Digest, Sha1};
use std::fs;
use std::path::PathBuf;
use futures::stream::{self, StreamExt};

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

pub async fn download_files(tasks: Vec<DownloadTask>, concurrency: usize) -> Result<(), anyhow::Error> {
    let client = Client::new();

    let fetches = stream::iter(tasks).map(|task| {
        let client = client.clone();
        tokio::spawn(async move {
            if check_file_validity(&task.path, &task.sha1) {
                // println!("Skipping already downloaded: {:?}", task.path);
                return Ok(());
            }

            if let Some(parent) = task.path.parent() {
                let _ = fs::create_dir_all(parent);
            }

            // println!("Downloading: {}", task.url);
            match client.get(&task.url).send().await {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        if let Ok(bytes) = response.bytes().await {
                            if let Err(e) = fs::write(&task.path, bytes) {
                                return Err(anyhow::anyhow!("Failed to write file {:?}: {}", task.path, e));
                            }
                            return Ok(());
                        }
                    }
                    Err(anyhow::anyhow!("Failed to download {} (Status: {})", task.url, status))
                }
                Err(e) => Err(anyhow::anyhow!("Request error for {}: {}", task.url, e)),
            }
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
