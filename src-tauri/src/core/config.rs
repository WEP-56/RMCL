use crate::models::account::Account;
use crate::models::settings::AppSettings;
use std::fs;
use std::path::PathBuf;

pub fn get_app_dir() -> PathBuf {
    let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("RustMCLauncher");
    if !path.exists() {
        fs::create_dir_all(&path).unwrap();
    }
    path
}

pub fn get_accounts_file() -> PathBuf {
    let mut path = get_app_dir();
    path.push("accounts.json");
    path
}

pub fn get_settings_file() -> PathBuf {
    let mut path = get_app_dir();
    path.push("settings.json");
    path
}

pub fn get_instances_dir() -> PathBuf {
    let mut path = get_app_dir();
    path.push("instances");
    if !path.exists() {
        fs::create_dir_all(&path).unwrap();
    }
    path
}

pub fn save_accounts(accounts: &Vec<Account>) -> Result<(), anyhow::Error> {
    let json = serde_json::to_string_pretty(accounts)?;
    fs::write(get_accounts_file(), json)?;
    Ok(())
}

pub fn load_accounts() -> Result<Vec<Account>, anyhow::Error> {
    let file_path = get_accounts_file();
    if !file_path.exists() {
        return Ok(Vec::new());
    }
    let data = fs::read_to_string(file_path)?;
    let accounts: Vec<Account> = serde_json::from_str(&data)?;
    Ok(accounts)
}

pub fn get_account_by_uuid(uuid: &str) -> Result<Account, anyhow::Error> {
    load_accounts()?
        .into_iter()
        .find(|account| account.uuid == uuid)
        .ok_or_else(|| anyhow::anyhow!("Account not found: {}", uuid))
}

pub fn save_settings(settings: &AppSettings) -> Result<(), anyhow::Error> {
    let json = serde_json::to_string_pretty(settings)?;
    fs::write(get_settings_file(), json)?;
    Ok(())
}

pub fn load_settings() -> Result<AppSettings, anyhow::Error> {
    let file_path = get_settings_file();
    if !file_path.exists() {
        return Ok(AppSettings::default());
    }
    if let Ok(data) = fs::read_to_string(file_path) {
        if let Ok(settings) = serde_json::from_str(&data) {
            return Ok(settings);
        }
    }
    Ok(AppSettings::default())
}
