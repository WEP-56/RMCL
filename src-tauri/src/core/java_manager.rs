use reqwest::Client;
use std::env::consts;
use std::process::Command;
use serde::{Serialize, Deserialize};
use std::path::PathBuf;

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
                
                let exe_name = if consts::OS == "windows" { "javaw.exe" } else { "java" };
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
    // Try to execute `java -version` and if it works, return "java"
    if Command::new("java").arg("-version").output().is_ok() {
        return "java".to_string();
    }

    // Try common Windows paths
    if consts::OS == "windows" {
        let common_paths = vec![
            "C:\\Program Files\\Java",
            "C:\\Program Files (x86)\\Java",
            "C:\\Program Files\\Eclipse Adoptium",
            "C:\\Program Files\\AdoptOpenJDK",
        ];
        
        for base in common_paths {
            if let Ok(entries) = std::fs::read_dir(base) {
                for entry in entries.flatten() {
                    let mut exe_path = entry.path();
                    exe_path.push("bin");
                    exe_path.push("javaw.exe");
                    if exe_path.exists() {
                        return exe_path.to_string_lossy().to_string();
                    }
                    
                    let mut java_exe = entry.path();
                    java_exe.push("bin");
                    java_exe.push("java.exe");
                    if java_exe.exists() {
                        return java_exe.to_string_lossy().to_string();
                    }
                }
            }
        }
    }

    // Try common Linux/Mac paths
    if consts::OS == "linux" {
        let common_paths = vec![
            "/usr/lib/jvm",
            "/usr/java",
        ];
        
        for base in common_paths {
            if let Ok(entries) = std::fs::read_dir(base) {
                for entry in entries.flatten() {
                    let mut exe_path = entry.path();
                    exe_path.push("bin");
                    exe_path.push("java");
                    if exe_path.exists() {
                        return exe_path.to_string_lossy().to_string();
                    }
                }
            }
        }
    }

    "java".to_string() // Fallback
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
