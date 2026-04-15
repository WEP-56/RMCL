use std::path::Path;
use tokio::process::Command;
use std::process::Stdio;
use tauri::Emitter;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::time::{sleep, Duration};

fn emit_and_log(app_handle: &tauri::AppHandle, log_path: &Path, line: String) {
    let _ = crate::core::launch_log::append_log_line(log_path, &line);
    let _ = app_handle.emit("mc-log", line);
}

pub async fn spawn_minecraft(
    app_handle: tauri::AppHandle,
    java_path: &str,
    args: Vec<String>,
    working_dir: &str,
    log_path: &Path,
) -> Result<(), anyhow::Error> {
    emit_and_log(&app_handle, log_path, format!("[INFO] Launching Java from: {}", java_path));
    emit_and_log(&app_handle, log_path, format!("[INFO] Working Directory: {}", working_dir));
    emit_and_log(&app_handle, log_path, format!("[INFO] Arguments: {:?}", args));
    println!("[INFO] Launching Java from: {}", java_path);
    println!("[INFO] Working Directory: {}", working_dir);
    println!("[INFO] Arguments length: {}", args.len());

    let mut child = Command::new(java_path)
        .args(&args)
        .current_dir(working_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // Don't wait for javaw.exe if it exits immediately. Actually javaw.exe detaches its streams
    // and returns immediately or closes pipes on windows. Wait, we should use java.exe instead of javaw.exe.
    
    // We will wait briefly to see if it crashed
    sleep(Duration::from_millis(500)).await;
    if let Some(status) = child.try_wait()? {
        // If it exited within 500ms, it's definitely a crash
        let code = status.code().unwrap_or(-1);
        let _ = crate::core::launch_log::append_log_line(
            log_path,
            &format!("[ERROR] Minecraft exited immediately with code {}", code),
        );
        let _ = app_handle.emit("mc-exit", code);
        
        return Err(anyhow::anyhow!(
            "Minecraft exited immediately (code {}). Please check if your java path is correct.",
            code
        ));
    }

    let stdout = child.stdout.take().expect("Failed to open stdout");
    let stderr = child.stderr.take().expect("Failed to open stderr");

    let app_clone1 = app_handle.clone();
    let stdout_log_path = log_path.to_path_buf();
    tokio::spawn(async move {
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let _ = crate::core::launch_log::append_log_line(&stdout_log_path, &line);
            let _ = app_clone1.emit("mc-log", line.clone());
            println!("[MC-STDOUT] {}", line);
        }
    });

    let app_clone2 = app_handle.clone();
    let stderr_log_path = log_path.to_path_buf();
    tokio::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let formatted = format!("[ERROR] {}", line);
            let _ = crate::core::launch_log::append_log_line(&stderr_log_path, &formatted);
            let _ = app_clone2.emit("mc-log", formatted);
            println!("[MC-STDERR] {}", line);
        }
    });

    let app_clone3 = app_handle.clone();
    let exit_log_path = log_path.to_path_buf();
    tokio::spawn(async move {
        // Wait for game to exit in a separate task so we don't block the Tauri command
        if let Ok(status) = child.wait().await {
            let exit_code = status.code().unwrap_or(-1);
            let _ = crate::core::launch_log::append_log_line(
                &exit_log_path,
                &format!("[INFO] Minecraft exited with code {}", exit_code),
            );
            let _ = app_clone3.emit("mc-exit", status.code().unwrap_or(-1));
        }
    });

    Ok(())
}
