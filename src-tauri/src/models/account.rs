use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccountType {
    Offline,
    Microsoft,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub uuid: String,
    pub username: String,
    pub account_type: AccountType,
    pub access_token: Option<String>,
}

impl Account {
    pub fn new_offline(username: String, uuid: String) -> Self {
        Self {
            uuid,
            username,
            account_type: AccountType::Offline,
            access_token: None,
        }
    }
}
