pub mod core;
pub mod models;

use crate::core::minecraft::VersionManifest;
use crate::models::account::Account;
use crate::models::instance::{Instance, LoaderType};
use crate::models::manifest::VersionMeta;
use crate::models::modrinth::{SearchResult, Version};
use std::collections::HashMap;

#[tauri::command]
async fn search_modrinth(query: String, limit: u32, offset: u32) -> Result<SearchResult, String> {
    core::modrinth_api::search_projects(&query, None, None, limit, offset)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_modrinth_versions(project_id: String) -> Result<Vec<Version>, String> {
    core::modrinth_api::get_project_versions(&project_id, None, None)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn install_mod(instance_id: String, version: Version) -> Result<(), String> {
    core::mod_manager::install_mod_version(&instance_id, &version)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_java_download_url(major_version: u32) -> Result<String, String> {
    core::java_manager::get_java_download_url(major_version)
        .await
        .map_err(|e| e.to_string())
}#[tauri::command]
async fn launch_minecraft(
    app: tauri::AppHandle,
    version_id: String,
    username: String,
    java_path: String,
) -> Result<(), String> {
    // 1. Fetch metadata
    let url = "https://piston-meta.mojang.com/v1/packages/8c198a22dbaa8c88939023405fa0cd9563fc8a26/1.20.1.json"; // Hardcoded for demo, should lookup from manifest
    let meta = core::resolver::fetch_version_meta(&version_id, url)
        .await
        .map_err(|e| e.to_string())?;

    // 2. Download libraries and client
    core::download_manager::download_libraries(&meta)
        .await
        .map_err(|e| e.to_string())?;
    core::download_manager::download_client_jar(&meta)
        .await
        .map_err(|e| e.to_string())?;

    // 3. Download Assets
    if let Some(asset_index) = &meta.asset_index {
        core::assets_manager::download_assets(&asset_index.url, &asset_index.id)
            .await
            .map_err(|e| e.to_string())?;
    }

    // 4. Extract Natives
    let natives_dir = core::natives_extractor::extract_natives(&meta).map_err(|e| e.to_string())?;

    // 5. Build Classpath
    let classpath = core::launcher::build_classpath(&meta);

    // 6. Build Placeholders
    let mut placeholders = HashMap::new();
    placeholders.insert("auth_player_name", username.clone());
    placeholders.insert("version_name", version_id.clone());
    placeholders.insert("game_directory", core::paths::get_minecraft_dir().to_string_lossy().to_string());
    placeholders.insert("assets_root", core::paths::get_assets_dir().to_string_lossy().to_string());
    placeholders.insert("assets_index_name", meta.asset_index.map_or("".to_string(), |a| a.id));
    placeholders.insert("auth_uuid", core::auth::generate_offline_uuid(&username));
    placeholders.insert("auth_access_token", "null".to_string());
    placeholders.insert("user_type", "mojang".to_string()); // or msa
    placeholders.insert("version_type", meta.version_type.clone());
    placeholders.insert("resolution_width", "854".to_string());
    placeholders.insert("resolution_height", "480".to_string());
    placeholders.insert("natives_directory", natives_dir.to_string_lossy().to_string());
    placeholders.insert("launcher_name", "RustMCLauncher".to_string());
    placeholders.insert("launcher_version", "0.1.0".to_string());
    placeholders.insert("classpath", classpath);

    // 7. Parse Arguments
    let mut final_args = Vec::new();
    
    // Add memory and Natives
    final_args.push("-Xmx2G".to_string());
    final_args.push("-XX:+UnlockExperimentalVMOptions".to_string());
    final_args.push("-XX:+UseG1GC".to_string());
    final_args.push("-XX:G1NewSizePercent=20".to_string());
    final_args.push("-XX:G1ReservePercent=20".to_string());
    final_args.push("-XX:MaxGCPauseMillis=50".to_string());
    final_args.push("-XX:G1HeapRegionSize=32M".to_string());
    final_args.push(format!("-Djava.library.path={}", natives_dir.to_string_lossy()));

    if let Some(args) = &meta.arguments {
        if let Some(jvm_args) = &args.jvm {
            final_args.extend(core::launcher::parse_arguments(jvm_args, &placeholders));
        }
        
        final_args.push(meta.main_class.clone());

        if let Some(game_args) = &args.game {
            final_args.extend(core::launcher::parse_arguments(game_args, &placeholders));
        }
    } else {
        // Fallback for old versions (1.12.2 and older)
        final_args.push(meta.main_class.clone());
        if let Some(mc_args) = &meta.minecraft_arguments {
            let parsed_old = mc_args.split_whitespace().map(|s| core::launcher::replace_placeholders(s, &placeholders)).collect::<Vec<String>>();
            final_args.extend(parsed_old);
        }
    }

    let working_dir = core::paths::get_minecraft_dir().to_string_lossy().to_string();

    // 8. Spawn process
    core::process_manager::spawn_minecraft(app, &java_path, final_args, &working_dir)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn get_version_meta(version_id: String, url: String) -> Result<VersionMeta, String> {
    core::resolver::fetch_version_meta(&version_id, &url).await.map_err(|e| e.to_string())
}

#[tauri::command]
fn add_offline_account(username: String) -> Result<Account, String> {    let account = core::auth::login_offline(username);
    let mut accounts = core::config::load_accounts().unwrap_or_else(|_| vec![]);

    // Check if account already exists
    if !accounts.iter().any(|a| a.uuid == account.uuid) {
        accounts.push(account.clone());
        core::config::save_accounts(&accounts).map_err(|e| e.to_string())?;
    }

    Ok(account)
}

#[tauri::command]
fn get_accounts() -> Result<Vec<Account>, String> {
    core::config::load_accounts().map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_instance(name: String, mc_version: String, loader: LoaderType, use_performance_preset: bool) -> Result<Instance, String> {
    let instance = core::instance::create_instance(name, mc_version.clone(), loader.clone()).map_err(|e| e.to_string())?;

    // If using Fabric, fetch its meta automatically
    if let LoaderType::Fabric = loader {
        if let Ok(loader_version) = core::fabric_manager::fetch_latest_fabric_loader(&mc_version).await {
            let _ = core::fabric_manager::fetch_fabric_meta(&mc_version, &loader_version).await;
            
            // If performance preset is requested, install sodium/iris/lithium
            if use_performance_preset {
                let _ = core::preset_manager::install_performance_preset(&instance.id, &mc_version).await;
            }
        }
    }

    Ok(instance)
}

#[tauri::command]
fn get_instances() -> Result<Vec<Instance>, String> {
    core::instance::load_instances().map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_instance(id: String) -> Result<(), String> {
    core::instance::delete_instance(&id).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_minecraft_versions() -> Result<crate::core::minecraft::VersionManifest, String> {
    core::minecraft::fetch_version_manifest().await.map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
      add_offline_account,
      get_accounts,
      create_instance,
      get_instances,
      delete_instance,
      get_minecraft_versions,
      get_version_meta,
      launch_minecraft,
      search_modrinth,
      get_modrinth_versions,
      install_mod,
      get_java_download_url
    ])
    .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
