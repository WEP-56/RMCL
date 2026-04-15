use crate::core::instance;
use serde::Serialize;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize)]
pub struct LaunchLogFile {
    pub file_name: String,
    pub modified_ms: u64,
    pub size: u64,
}

pub fn create_launch_log(instance_id: &str) -> Result<PathBuf, anyhow::Error> {
    let logs_dir = instance::get_instance_logs_dir(instance_id);
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| anyhow::anyhow!("System clock error: {}", e))?
        .as_millis();
    let log_path = logs_dir.join(format!("launch-{}.log", timestamp));
    append_log_line(&log_path, "[INFO] Launch log created")?;
    Ok(log_path)
}

pub fn append_log_line(log_path: &Path, line: &str) -> Result<(), anyhow::Error> {
    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;
    writeln!(file, "{}", line)?;
    Ok(())
}

pub fn list_launch_logs(instance_id: &str) -> Result<Vec<LaunchLogFile>, anyhow::Error> {
    let logs_dir = instance::get_instance_logs_dir(instance_id);
    let mut logs = Vec::new();

    for entry in fs::read_dir(logs_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !file_name.ends_with(".log") {
            continue;
        }

        let metadata = entry.metadata()?;
        let modified_ms = metadata
            .modified()
            .ok()
            .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_millis() as u64)
            .unwrap_or(0);

        logs.push(LaunchLogFile {
            file_name: file_name.to_string(),
            modified_ms,
            size: metadata.len(),
        });
    }

    logs.sort_by(|left, right| {
        right
            .modified_ms
            .cmp(&left.modified_ms)
            .then_with(|| right.file_name.cmp(&left.file_name))
    });

    Ok(logs)
}

pub fn read_launch_log(instance_id: &str, file_name: &str) -> Result<String, anyhow::Error> {
    let safe_name = Path::new(file_name)
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid log file name"))?;
    let log_path = instance::get_instance_logs_dir(instance_id).join(safe_name);
    Ok(fs::read_to_string(log_path)?)
}
