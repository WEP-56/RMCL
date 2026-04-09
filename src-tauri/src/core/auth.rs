use crate::models::account::Account;
use uuid::Uuid;

// Offline UUIDs follow UUID v3 specification based on the player name
pub fn generate_offline_uuid(username: &str) -> String {
    let namespace = Uuid::parse_str("OfflinePlayer:").unwrap_or_else(|_| Uuid::nil());
    let formatted_name = format!("OfflinePlayer:{}", username);
    Uuid::new_v3(&namespace, formatted_name.as_bytes()).to_string()
}

pub fn login_offline(username: String) -> Account {
    let uuid = generate_offline_uuid(&username);
    Account::new_offline(username, uuid)
}
