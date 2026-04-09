use crate::models::account::Account;
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
