use tokio::process::Command;
use std::process::Stdio;
use tauri::Emitter;
use tokio::io::{AsyncBufReadExt, BufReader};

pub async fn spawn_minecraft(
    app_handle: tauri::AppHandle,
    java_path: &str,
    args: Vec<String>,
    working_dir: &str,
) -> Result<(), anyhow::Error> {
    let mut child = Command::new(java_path)
        .args(args)
        .current_dir(working_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().expect("Failed to open stdout");
    let stderr = child.stderr.take().expect("Failed to open stderr");

    let app_clone1 = app_handle.clone();
    tokio::spawn(async move {
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let _ = app_clone1.emit("mc-log", line);
        }
    });

    let app_clone2 = app_handle.clone();
    tokio::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let _ = app_clone2.emit("mc-log", format!("[ERROR] {}", line));
        }
    });

    let app_clone3 = app_handle.clone();
    tokio::spawn(async move {
        // Wait for game to exit in a separate task so we don't block the Tauri command
        if let Ok(status) = child.wait().await {
            let _ = app_clone3.emit("mc-exit", status.code().unwrap_or(-1));
        }
    });

    Ok(())
}
