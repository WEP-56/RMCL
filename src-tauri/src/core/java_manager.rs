use reqwest::Client;
use std::env::consts;

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
