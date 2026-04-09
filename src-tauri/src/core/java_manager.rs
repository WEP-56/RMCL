use reqwest::Client;
use std::env::consts;
use std::process::Command;

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
