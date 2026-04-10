use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::models::account::{Account, AccountType};
use std::time::Duration;
use tokio::time::sleep;
use log::info;

const CLIENT_ID: &str = "c36a9fb6-4f2a-41ff-90bd-ae7cc92031eb";

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DeviceCodeResponse {
    pub user_code: String,
    pub device_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
    pub message: String,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
}

#[derive(Serialize)]
struct XblAuthRequest {
    Properties: XblProperties,
    RelyingParty: String,
    TokenType: String,
}

#[derive(Serialize)]
struct XblProperties {
    AuthMethod: String,
    SiteName: String,
    RpsTicket: String,
}

#[derive(Deserialize)]
struct XblResponse {
    Token: String,
    DisplayClaims: XblDisplayClaims,
}

#[derive(Deserialize)]
struct XblDisplayClaims {
    xui: Vec<XblXui>,
}

#[derive(Deserialize)]
struct XblXui {
    uhs: String,
}

#[derive(Serialize)]
struct XstsAuthRequest {
    Properties: XstsProperties,
    RelyingParty: String,
    TokenType: String,
}

#[derive(Serialize)]
struct XstsProperties {
    SandboxId: String,
    UserTokens: Vec<String>,
}

#[derive(Deserialize)]
struct XstsResponse {
    Token: String,
}

#[derive(Serialize)]
struct McAuthRequest {
    identityToken: String,
}

#[derive(Deserialize)]
struct McAuthResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct McProfileResponse {
    id: String,
    name: String,
}

pub async fn start_device_code_flow() -> Result<DeviceCodeResponse, String> {
    let client = Client::new();
    let res = client
        .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode")
        .form(&[
            ("client_id", CLIENT_ID),
            ("scope", "XboxLive.signin offline_access"),
        ])
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let data: DeviceCodeResponse = res.json().await.map_err(|e| e.to_string())?;
    Ok(data)
}

pub async fn poll_msa_token(device_code: String, interval: u64) -> Result<Account, String> {
    let client = Client::new();
    
    // Poll for OAuth token
    let mut oauth_token = String::new();
    loop {
        let res = client
            .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/token")
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ("client_id", CLIENT_ID),
                ("device_code", &device_code),
            ])
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if res.status().is_success() {
            let data: TokenResponse = res.json().await.map_err(|e| e.to_string())?;
            oauth_token = data.access_token;
            break;
        } else {
            let text = res.text().await.unwrap_or_default();
            if text.contains("authorization_pending") {
                sleep(Duration::from_secs(interval)).await;
            } else {
                return Err(format!("MSA Error: {}", text));
            }
        }
    }

    info!("Got MSA token, starting XBL auth");

    // XBL Auth
    let xbl_req = XblAuthRequest {
        Properties: XblProperties {
            AuthMethod: "RPS".into(),
            SiteName: "user.auth.xboxlive.com".into(),
            RpsTicket: format!("d={}", oauth_token),
        },
        RelyingParty: "http://auth.xboxlive.com".into(),
        TokenType: "JWT".into(),
    };

    let xbl_res = client
        .post("https://user.auth.xboxlive.com/user/authenticate")
        .json(&xbl_req)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let xbl_data: XblResponse = xbl_res.json().await.map_err(|e| e.to_string())?;
    let xbl_token = xbl_data.Token;
    let uhs = xbl_data.DisplayClaims.xui[0].uhs.clone();

    info!("Got XBL token, starting XSTS auth");

    // XSTS Auth
    let xsts_req = XstsAuthRequest {
        Properties: XstsProperties {
            SandboxId: "RETAIL".into(),
            UserTokens: vec![xbl_token],
        },
        RelyingParty: "rp://api.minecraftservices.com/".into(),
        TokenType: "JWT".into(),
    };

    let xsts_res = client
        .post("https://xsts.auth.xboxlive.com/xsts/authorize")
        .json(&xsts_req)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !xsts_res.status().is_success() {
        return Err("Xbox account does not have Minecraft or age restricted.".into());
    }

    let xsts_data: XstsResponse = xsts_res.json().await.map_err(|e| e.to_string())?;
    let xsts_token = xsts_data.Token;

    info!("Got XSTS token, logging into Minecraft");

    // Minecraft Auth
    let mc_req = McAuthRequest {
        identityToken: format!("XBL3.0 x={};{}", uhs, xsts_token),
    };

    let mc_res = client
        .post("https://api.minecraftservices.com/authentication/login_with_xbox")
        .json(&mc_req)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let mc_data: McAuthResponse = mc_res.json().await.map_err(|e| e.to_string())?;
    let mc_token = mc_data.access_token;

    // Get Profile
    let profile_res = client
        .get("https://api.minecraftservices.com/minecraft/profile")
        .header("Authorization", format!("Bearer {}", mc_token))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let profile_data: McProfileResponse = profile_res.json().await.map_err(|e| e.to_string())?;

    // Create and return account
    let mut account = Account::new_offline(profile_data.name, profile_data.id);
    account.account_type = AccountType::Microsoft;
    account.access_token = Some(mc_token);

    Ok(account)
}
