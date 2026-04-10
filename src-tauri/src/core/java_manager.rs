use reqwest::Client;
use std::env::consts;
use std::process::Command;
use std::path::PathBuf;
use std::fs::{self, File};
use std::io::Write;
use serde::{Serialize, Deserialize};
use tauri::Emitter;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JavaInstallation {
    pub name: String,
    pub path: String,
    pub version: String,
}

pub fn get_java_version_from_path(path: &str) -> Option<String> {
    let output = Command::new(path).arg("-version").output().ok()?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Some java distributions print to stdout, most to stderr
    let output_str = if !stderr.trim().is_empty() { stderr } else { stdout };
    
    // Look for a line containing "version"
    for line in output_str.lines() {
        if line.contains("version") {
            // Extract text between quotes
            if let Some(start) = line.find('"') {
                if let Some(end) = line[start + 1..].find('"') {
                    return Some(line[start + 1..start + 1 + end].to_string());
                }
            }
            // Fallback if no quotes
            return Some(line.to_string());
        }
    }
    None
}

pub fn scan_java_installations() -> Vec<JavaInstallation> {
    let mut installations = Vec::new();
    
    // Try system 'java'
    if let Some(version) = get_java_version_from_path("java") {
        installations.push(JavaInstallation {
            name: "System Default".to_string(),
            path: "java".to_string(),
            version,
        });
    }

    let mut search_paths = Vec::new();

    if consts::OS == "windows" {
        search_paths.extend(vec![
            "C:\\Program Files\\Java",
            "C:\\Program Files (x86)\\Java",
            "C:\\Program Files\\Eclipse Adoptium",
            "C:\\Program Files\\AdoptOpenJDK",
        ]);
    } else if consts::OS == "linux" {
        search_paths.extend(vec![
            "/usr/lib/jvm",
            "/usr/java",
        ]);
    } else if consts::OS == "macos" {
        search_paths.extend(vec![
            "/Library/Java/JavaVirtualMachines",
        ]);
    }

    for base in search_paths {
        if let Ok(entries) = std::fs::read_dir(base) {
            for entry in entries.flatten() {
                let mut exe_path = entry.path();
                exe_path.push("bin");
                
                let exe_name = if consts::OS == "windows" { "java.exe" } else { "java" };
                exe_path.push(exe_name);
                
                if exe_path.exists() {
                    let path_str = exe_path.to_string_lossy().to_string();
                    if let Some(version) = get_java_version_from_path(&path_str) {
                        let name = entry.file_name().to_string_lossy().to_string();
                        // Avoid duplicates
                        if !installations.iter().any(|i| i.path == path_str || (i.path == "java" && i.version == version)) {
                            installations.push(JavaInstallation {
                                name,
                                path: path_str,
                                version,
                            });
                        }
                    }
                }
            }
        }
    }

    installations
}

pub fn find_system_java() -> String {
    let installations = scan_java_installations();
    if !installations.is_empty() {
        return installations[0].path.clone();
    }
    "java".to_string()
}

pub fn find_java_by_major_version(major_version: u32) -> Option<String> {
    let installations = scan_java_installations();
    for java in installations {
        // basic version string parse like "1.8.0_302", "17.0.1", "21"
        let parts: Vec<&str> = java.version.split(|c| c == '.' || c == '_' || c == '-').collect();
        if parts.is_empty() { continue; }
        
        let parsed_major = if parts[0] == "1" && parts.len() > 1 {
            // 1.8.0 -> 8
            parts[1].parse::<u32>().unwrap_or(0)
        } else {
            // 17.0.1 -> 17
            parts[0].parse::<u32>().unwrap_or(0)
        };

        if parsed_major == major_version {
            return Some(java.path);
        }
    }
    None
}

const ADOPTIUM_API_URL: &str = "https://api.adoptium.net/v3";

pub fn get_os_arch_strings() -> (&'static str, &'static str) {
    let os = match consts::OS {
        "windows" => "windows",
        "macos" => "mac",
        "linux" => "linux",
        _ => "linux", // Fallback
    };

    let arch = match consts::ARCH {
        "x86_64" => "x64",
        "aarch64" => "aarch64",
        "x86" => "x32",
        _ => "x64", // Fallback
    };

    (os, arch)
}

pub async fn get_java_download_url(major_version: u32) -> Result<String, anyhow::Error> {
    let (os, arch) = get_os_arch_strings();
    
    // Using JRE instead of JDK to save space
    let url = format!(
        "{}/binary/latest/{}/ga/{}/{}/jre/hotspot/normal/eclipse",
        ADOPTIUM_API_URL, major_version, os, arch
    );
    
    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none()) // Don't follow redirect, just get the URL
        .build()?;
        
    let response = client.get(&url).send().await?;
    
    // The Adoptium API returns a 302 Found with the actual download link in the Location header
    if response.status().is_redirection() {
        if let Some(location) = response.headers().get("Location") {
            return Ok(location.to_str()?.to_string());
        }
    }
    
    // If not redirected, we can't get the direct link this way easily. 
    // Fallback: the URL itself will redirect the user's browser/downloader.
    Ok(url)
}

pub async fn download_and_extract_java(
    major_version: u32,
    app: tauri::AppHandle,
    instance_id: &str,
) -> Result<String, anyhow::Error> {
    let url = get_java_download_url(major_version).await?;
    let _ = app.emit("mc-progress", crate::core::downloader::ProgressPayload {
        instance_id: instance_id.to_string(),
        task: format!("下载 Java {}...", major_version),
        progress: -1.0,
        text: "正在下载 JRE".to_string(),
    });

    let java_dir = crate::core::config::get_instances_dir().parent().unwrap().join("java");
    if !java_dir.exists() {
        fs::create_dir_all(&java_dir)?;
    }

    let target_dir = java_dir.join(major_version.to_string());
    if target_dir.exists() {
        // Find existing java executable
        if let Some(exe) = find_java_executable_in_dir(&target_dir) {
            return Ok(exe);
        }
    }

    let is_zip = url.ends_with(".zip") || consts::OS == "windows";
    let temp_file_path = java_dir.join(format!("java_{}.tmp", major_version));

    // Download
    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()?;
    let response = client.get(&url).send().await?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("Failed to download Java from {}: {}", url, response.status()));
    }

    let mut bytes = response.bytes().await?;
    let mut file = File::create(&temp_file_path)?;
    file.write_all(&bytes)?;

    let _ = app.emit("mc-progress", crate::core::downloader::ProgressPayload {
        instance_id: instance_id.to_string(),
        task: format!("解压 Java {}...", major_version),
        progress: -1.0,
        text: "解压中...".to_string(),
    });

    // Extract
    fs::create_dir_all(&target_dir)?;
    if is_zip {
        let zip_file = File::open(&temp_file_path)?;
        let mut archive = zip::ZipArchive::new(zip_file)?;
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = match file.enclosed_name() {
                Some(path) => target_dir.join(path),
                None => continue,
            };

            if file.is_dir() {
                fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    fs::create_dir_all(p)?;
                }
                let mut outfile = File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
            
            // Set permissions on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = file.unix_mode() {
                    let _ = fs::set_permissions(&outpath, fs::Permissions::from_mode(mode));
                }
            }
        }
    } else {
        // Tar Gz
        let tar_gz = File::open(&temp_file_path)?;
        let tar = flate2::read::GzDecoder::new(tar_gz);
        let mut archive = tar::Archive::new(tar);
        archive.unpack(&target_dir)?;
    }

    // Cleanup
    let _ = fs::remove_file(&temp_file_path);

    if let Some(exe) = find_java_executable_in_dir(&target_dir) {
        // On Unix, ensure it is executable (in case tar didn't set it)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&exe)?.permissions();
            perms.set_mode(0o755);
            let _ = fs::set_permissions(&exe, perms);
        }
        return Ok(exe);
    }

    Err(anyhow::anyhow!("Failed to locate java executable in extracted directory"))
}

fn find_java_executable_in_dir(dir: &PathBuf) -> Option<String> {
    let walk_dir = walkdir::WalkDir::new(dir);
    let exe_name = if consts::OS == "windows" { "java.exe" } else { "java" };
    
    for entry in walk_dir.into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.file_name().unwrap_or_default() == exe_name {
            // make sure it's in a 'bin' directory to avoid finding other files named java
            if let Some(parent) = path.parent() {
                if parent.file_name().unwrap_or_default() == "bin" {
                    return Some(path.to_string_lossy().to_string());
                }
            }
        }
    }
    None
}
