use crate::models::account::{Account, AccountType};
use std::collections::HashMap;
use std::path::Path;

pub const LAUNCHER_NAME: &str = "RustMCLauncher";
pub const LAUNCHER_VERSION: &str = "0.1.0";

fn resolve_auth_uuid(account: &Account) -> String {
    if account.uuid.trim().is_empty() {
        crate::core::auth::generate_offline_uuid(&account.username)
    } else {
        account.uuid.clone()
    }
}

fn resolve_auth_access_token(account: &Account) -> Result<String, anyhow::Error> {
    match account.account_type {
        AccountType::Offline => Ok("null".to_string()),
        AccountType::Microsoft => account
            .access_token
            .clone()
            .filter(|token| !token.trim().is_empty())
            .ok_or_else(|| anyhow::anyhow!("Selected Microsoft account is missing an access token. Please sign in again.")),
    }
}

fn resolve_user_type(account: &Account) -> &'static str {
    match account.account_type {
        AccountType::Offline => "legacy",
        AccountType::Microsoft => "msa",
    }
}

pub fn build_default_placeholders(
    account: &Account,
    version_id: &str,
    version_type: &str,
    game_dir: &Path,
    assets_root: &Path,
    assets_index_name: Option<&str>,
    natives_dir: &Path,
    classpath: &str,
) -> Result<HashMap<String, String>, anyhow::Error> {
    let mut placeholders = HashMap::with_capacity(20);

    placeholders.insert("auth_player_name".to_string(), account.username.clone());
    placeholders.insert("version_name".to_string(), version_id.to_string());
    placeholders.insert(
        "game_directory".to_string(),
        game_dir.to_string_lossy().to_string(),
    );
    placeholders.insert(
        "assets_root".to_string(),
        assets_root.to_string_lossy().to_string(),
    );
    placeholders.insert(
        "assets_index_name".to_string(),
        assets_index_name.unwrap_or_default().to_string(),
    );
    placeholders.insert(
        "auth_uuid".to_string(),
        resolve_auth_uuid(account),
    );
    placeholders.insert(
        "auth_access_token".to_string(),
        resolve_auth_access_token(account)?,
    );
    placeholders.insert("user_type".to_string(), resolve_user_type(account).to_string());
    placeholders.insert("version_type".to_string(), version_type.to_string());
    placeholders.insert(
        "natives_directory".to_string(),
        natives_dir.to_string_lossy().to_string(),
    );
    placeholders.insert("launcher_name".to_string(), LAUNCHER_NAME.to_string());
    placeholders.insert("launcher_version".to_string(), LAUNCHER_VERSION.to_string());
    placeholders.insert("classpath".to_string(), classpath.to_string());
    placeholders.insert("clientid".to_string(), "null".to_string());
    placeholders.insert("auth_xuid".to_string(), "null".to_string());
    placeholders.insert("user_properties".to_string(), "{}".to_string());
    placeholders.insert("resolution_width".to_string(), "854".to_string());
    placeholders.insert("resolution_height".to_string(), "480".to_string());

    Ok(placeholders)
}

pub fn replace_in_text(text: &str, placeholders: &HashMap<String, String>) -> String {
    let mut result = text.to_string();
    for (key, value) in placeholders {
        let token = format!("${{{}}}", key);
        result = result.replace(&token, value);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::account::{Account, AccountType};
    use std::path::PathBuf;

    fn offline_account() -> Account {
        Account::new_offline("Steve".to_string(), "offline-uuid".to_string())
    }

    #[test]
    fn build_default_placeholders_includes_required_keys() {
        let game_dir = PathBuf::from("C:/tmp/game");
        let assets_root = PathBuf::from("C:/tmp/assets");
        let natives_dir = PathBuf::from("C:/tmp/natives");
        let placeholders = build_default_placeholders(
            &offline_account(),
            "1.20.1",
            "release",
            &game_dir,
            &assets_root,
            Some("1.20"),
            &natives_dir,
            "a.jar;b.jar",
        )
        .expect("offline account placeholders should build");

        for key in [
            "auth_player_name",
            "version_name",
            "game_directory",
            "assets_root",
            "assets_index_name",
            "auth_uuid",
            "auth_access_token",
            "user_type",
            "version_type",
            "natives_directory",
            "launcher_name",
            "launcher_version",
            "classpath",
            "clientid",
            "auth_xuid",
        ] {
            assert!(
                placeholders.contains_key(key),
                "missing placeholder key: {key}"
            );
        }

        assert_eq!(
            placeholders.get("auth_player_name").map(String::as_str),
            Some("Steve")
        );
        assert_eq!(
            placeholders.get("auth_uuid").map(String::as_str),
            Some("offline-uuid")
        );
        assert_eq!(
            placeholders.get("user_type").map(String::as_str),
            Some("legacy")
        );
        assert_eq!(
            placeholders.get("assets_index_name").map(String::as_str),
            Some("1.20")
        );
        assert_eq!(
            placeholders.get("version_type").map(String::as_str),
            Some("release")
        );
    }

    #[test]
    fn build_default_placeholders_uses_microsoft_session_values() {
        let account = Account {
            uuid: "msa-uuid".to_string(),
            username: "Alex".to_string(),
            account_type: AccountType::Microsoft,
            access_token: Some("msa-token".to_string()),
        };

        let placeholders = build_default_placeholders(
            &account,
            "1.20.1",
            "release",
            Path::new("C:/tmp/game"),
            Path::new("C:/tmp/assets"),
            Some("1.20"),
            Path::new("C:/tmp/natives"),
            "cp.jar",
        )
        .expect("microsoft account placeholders should build");

        assert_eq!(
            placeholders.get("auth_player_name").map(String::as_str),
            Some("Alex")
        );
        assert_eq!(
            placeholders.get("auth_uuid").map(String::as_str),
            Some("msa-uuid")
        );
        assert_eq!(
            placeholders.get("auth_access_token").map(String::as_str),
            Some("msa-token")
        );
        assert_eq!(
            placeholders.get("user_type").map(String::as_str),
            Some("msa")
        );
    }

    #[test]
    fn build_default_placeholders_rejects_microsoft_account_without_token() {
        let account = Account {
            uuid: "msa-uuid".to_string(),
            username: "Alex".to_string(),
            account_type: AccountType::Microsoft,
            access_token: None,
        };

        let error = build_default_placeholders(
            &account,
            "1.20.1",
            "release",
            Path::new("C:/tmp/game"),
            Path::new("C:/tmp/assets"),
            Some("1.20"),
            Path::new("C:/tmp/natives"),
            "cp.jar",
        )
        .expect_err("missing microsoft token should fail");

        assert!(
            error.to_string().contains("access token"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn replace_in_text_replaces_all_known_tokens() {
        let mut placeholders = HashMap::new();
        placeholders.insert("version_name".to_string(), "1.20.1".to_string());
        placeholders.insert("auth_player_name".to_string(), "Alex".to_string());

        let replaced = replace_in_text(
            "--version ${version_name} --username ${auth_player_name}",
            &placeholders,
        );
        assert_eq!(replaced, "--version 1.20.1 --username Alex");
    }

    #[test]
    fn replace_in_text_keeps_unknown_tokens() {
        let placeholders = HashMap::new();
        let replaced = replace_in_text("--foo ${unknown}", &placeholders);
        assert_eq!(replaced, "--foo ${unknown}");
    }
}
