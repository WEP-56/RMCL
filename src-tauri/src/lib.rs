pub mod core;
pub mod models;

use crate::models::account::Account;
use crate::models::instance::{Instance, LoaderType};
use crate::models::manifest::VersionMeta;
use crate::models::modrinth::{SearchResult, Version};
use crate::models::settings::AppSettings;
use std::collections::HashMap;

use tauri::Emitter;

use crate::core::java_manager::JavaInstallation;

use crate::core::mod_manager::LocalMod;

#[tauri::command]
async fn export_instance(instance_id: String, output_path: String) -> Result<(), String> {
    crate::core::mod_manager::export_instance_to_modrinth(&instance_id, &output_path).await.map_err(|e| e.to_string())
}

#[tauri::command]
fn get_local_mods(instance_id: String) -> Result<Vec<LocalMod>, String> {
    crate::core::mod_manager::get_local_mods(&instance_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn toggle_mod(instance_id: String, mod_name: String, enabled: bool) -> Result<(), String> {
    crate::core::mod_manager::toggle_mod(&instance_id, &mod_name, enabled).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_mod(instance_id: String, mod_name: String) -> Result<(), String> {
    crate::core::mod_manager::delete_mod(&instance_id, &mod_name).map_err(|e| e.to_string())
}

#[tauri::command]
fn open_instance_folder(instance_id: String) -> Result<(), String> {
    crate::core::mod_manager::open_instance_folder(&instance_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn scan_java_installations() -> Result<Vec<JavaInstallation>, String> {
    Ok(crate::core::java_manager::scan_java_installations())
}

#[tauri::command]
async fn start_msa_login() -> Result<crate::core::msa::DeviceCodeResponse, String> {
    crate::core::msa::start_device_code_flow().await
}

#[tauri::command]
async fn poll_msa_token(device_code: String, interval: u64) -> Result<Account, String> {
    let account = crate::core::msa::poll_msa_token(device_code, interval).await?;
    let mut accounts = core::config::load_accounts().unwrap_or_else(|_| vec![]);

    // Check if account already exists
    if let Some(existing) = accounts.iter_mut().find(|a| a.uuid == account.uuid) {
        *existing = account.clone();
    } else {
        accounts.push(account.clone());
    }
    
    let _ = core::config::save_accounts(&accounts);

    Ok(account)
}

#[tauri::command]
async fn search_modrinth(query: String, project_type: Option<String>, limit: u32, offset: u32) -> Result<SearchResult, String> {
    core::modrinth_api::search_projects(&query, None, None, project_type.as_deref(), limit, offset)
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
async fn install_mod(app: tauri::AppHandle, instance_id: String, version: Version) -> Result<(), String> {
    core::mod_manager::install_mod_version(&instance_id, &version, Some(app))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn install_modpack(app: tauri::AppHandle, name: String, version: Version) -> Result<Instance, String> {
    core::mod_manager::install_modpack(&name, &version, Some(app))
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
    instance_id: String,
    username: String,
    java_path: String,
) -> Result<(), String> {
    // 0. Get Instance info
    let _ = app.emit("mc-log", format!("[INFO] 正在获取实例配置: {}...", instance_id));
    let instance = core::instance::get_instance_by_id(&instance_id).map_err(|e| e.to_string())?;
    let version_id = instance.mc_version.clone();
    
    let _ = app.emit("mc-log", "[INFO] 正在获取 Minecraft 版本清单...");
    let manifest = core::minecraft::fetch_version_manifest().await.map_err(|e| e.to_string())?;
    let version_info = manifest.versions.iter().find(|v| v.id == version_id)
        .ok_or_else(|| format!("Version {} not found in mojang manifest", version_id))?;
    
    let url = &version_info.url;
    let mut meta = core::resolver::fetch_version_meta(&version_id, url)
        .await
        .map_err(|e| e.to_string())?;

    // Resolve Java path based on version requirement
    let _ = app.emit("mc-log", "[INFO] 正在探测本地 Java 运行环境...");
    let mut resolved_java_path = java_path;
    if resolved_java_path == "java" {
        if let Some(java_req) = &meta.java_version {
            let _ = app.emit("mc-log", format!("[INFO] 游戏版本要求 Java 主版本: {}", java_req.major_version));
            if let Some(matched_java) = core::java_manager::find_java_by_major_version(java_req.major_version) {
                resolved_java_path = matched_java;
                let _ = app.emit("mc-log", format!("[INFO] 匹配到合适的 Java: {}", resolved_java_path));
            } else {
                let _ = app.emit("mc-log", format!("[WARN] 未能找到符合要求 (Java {}) 的本地运行环境，正在尝试自动下载...", java_req.major_version));
                match core::java_manager::download_and_extract_java(java_req.major_version, app.clone(), &instance_id).await {
                    Ok(downloaded_java) => {
                        resolved_java_path = downloaded_java;
                        let _ = app.emit("mc-log", format!("[INFO] 成功下载并配置了 Java: {}", resolved_java_path));
                    }
                    Err(e) => {
                        let _ = app.emit("mc-log", format!("[ERROR] 自动下载 Java 失败: {}。将尝试使用系统默认 Java", e));
                        resolved_java_path = core::java_manager::find_system_java();
                    }
                }
            }
        } else {
            resolved_java_path = core::java_manager::find_system_java();
        }
    }
    
    let _ = app.emit("mc-progress", core::downloader::ProgressPayload {
        instance_id: instance_id.clone(),
        task: "正在准备环境...".to_string(),
        progress: -1.0,
        text: "探测完成".to_string(),
    });

    let _ = app.emit("mc-progress", core::downloader::ProgressPayload {
        instance_id: instance_id.clone(),
        task: "合并 Fabric 库...".to_string(),
        progress: -1.0,
        text: "".to_string(),
    });

    // 1.5 If Fabric, merge the Fabric meta
    if let LoaderType::Fabric = instance.loader {
        let _ = app.emit("mc-log", "[INFO] 检测到 Fabric，正在合并核心库...");
        if let Some(loader_ver) = &instance.loader_version {
            let fabric_meta = core::fabric_manager::fetch_fabric_meta(&version_id, loader_ver).await.map_err(|e| e.to_string())?;
            // Simple merge: add fabric libraries to vanilla meta
            let mut fabric_libs = fabric_meta.libraries;
            meta.libraries.append(&mut fabric_libs);

            // Use fabric's main class and args
            meta.main_class = fabric_meta.main_class;
            // Merge arguments
            if let Some(mut vanilla_args) = meta.arguments.take() {
                if let Some(mut fabric_args) = fabric_meta.arguments {
                    if let Some(ref mut v_game) = vanilla_args.game {
                        if let Some(mut f_game) = fabric_args.game.take() {
                            // vanilla game args usually go first
                            v_game.append(&mut f_game);
                        }
                    } else {
                        vanilla_args.game = fabric_args.game.take();
                    }
                    if let Some(ref mut v_jvm) = vanilla_args.jvm {
                        if let Some(mut f_jvm) = fabric_args.jvm.take() {
                            // Fabric JVM args MUST go before vanilla JVM args to set the correct main class
                            f_jvm.append(v_jvm);
                            vanilla_args.jvm = Some(f_jvm);
                        }
                    } else {
                        vanilla_args.jvm = fabric_args.jvm.take();
                    }
                }
                meta.arguments = Some(vanilla_args);
            } else {
                meta.arguments = fabric_meta.arguments;
            }
        }
    }

    // 2. Download libraries and client
    core::download_manager::download_libraries(&meta, Some(app.clone()), &instance_id)
        .await
        .map_err(|e: anyhow::Error| e.to_string())?;
    core::download_manager::download_client_jar(&meta, Some(app.clone()), &instance_id)
        .await
        .map_err(|e: anyhow::Error| e.to_string())?;

    // 3. Download Assets
    if let Some(asset_index) = &meta.asset_index {
        core::assets_manager::download_assets(&asset_index.url, &asset_index.id, Some(app.clone()), &instance_id)
            .await
            .map_err(|e| e.to_string())?;
    }

    // 4. Extract Natives
    let _ = app.emit("mc-progress", core::downloader::ProgressPayload {
        instance_id: instance_id.clone(),
        task: "解压原生库...".to_string(),
        progress: -1.0,
        text: "提取 Natives 中".to_string(),
    });
    let natives_dir = core::natives_extractor::extract_natives(&meta).map_err(|e| e.to_string())?;

    // 5. Build Classpath
    let _ = app.emit("mc-progress", core::downloader::ProgressPayload {
        instance_id: instance_id.clone(),
        task: "构建启动参数...".to_string(),
        progress: -1.0,
        text: "生成 Classpath".to_string(),
    });
    let classpath = core::launcher::build_classpath(&meta);
    
    // Log classpath length to terminal to help debug issues
    println!("[INFO] Classpath generated with length: {}", classpath.len());

    // 6. Build Placeholders
    let mut placeholders = HashMap::new();
    let game_dir = core::instance::get_instance_game_dir(&instance_id);

    placeholders.insert("auth_player_name", username.clone());
    placeholders.insert("version_name", version_id.clone());
    placeholders.insert("game_directory", game_dir.to_string_lossy().to_string());
    placeholders.insert("assets_root", core::paths::get_assets_dir().to_string_lossy().to_string());
    placeholders.insert("assets_index_name", meta.asset_index.map_or("".to_string(), |a| a.id));
    placeholders.insert("auth_uuid", core::auth::generate_offline_uuid(&username));
    placeholders.insert("auth_access_token", "null".to_string());
    placeholders.insert("user_type", "mojang".to_string()); // or msa
    placeholders.insert("clientid", "null".to_string());
    placeholders.insert("auth_xuid", "null".to_string());
    placeholders.insert("user_properties", "{}".to_string());
    placeholders.insert("version_type", meta.version_type.clone());
    placeholders.insert("resolution_width", "854".to_string());
    placeholders.insert("resolution_height", "480".to_string());
    placeholders.insert("natives_directory", natives_dir.to_string_lossy().to_string());
    placeholders.insert("launcher_name", "RustMCLauncher".to_string());
    placeholders.insert("launcher_version", "0.1.0".to_string());
    placeholders.insert("classpath", classpath.clone());

    // 7. Parse Arguments
    let mut final_args = Vec::new();
    
    // Get max memory from settings
    let max_memory = core::config::load_settings().map(|s| s.max_memory).unwrap_or(2048);

    // Add memory and Natives
    final_args.push(format!("-Xmx{}M", max_memory));
    
    #[cfg(target_os = "macos")]
    final_args.push("-XstartOnFirstThread".to_string());
    
    final_args.push("-XX:+UnlockExperimentalVMOptions".to_string());
    final_args.push("-XX:+UseG1GC".to_string());
    final_args.push("-XX:G1NewSizePercent=20".to_string());
    final_args.push("-XX:G1ReservePercent=20".to_string());
    final_args.push("-XX:MaxGCPauseMillis=50".to_string());
    final_args.push("-XX:G1HeapRegionSize=32M".to_string());
    final_args.push(format!("-Djava.library.path={}", natives_dir.to_string_lossy()));

    let mut active_features = HashMap::new();
    active_features.insert("is_demo_user".to_string(), false);
    active_features.insert("has_custom_resolution".to_string(), true);
    active_features.insert("has_quick_plays_support".to_string(), false);

    // For modern versions
    if let Some(args) = &meta.arguments {
        if let Some(jvm_args) = &args.jvm {
            let parsed_jvm_args = core::launcher::parse_arguments(jvm_args, &placeholders, &active_features);
            
            // Strip out `-XstartOnFirstThread` from json arguments if not on macos
            #[cfg(not(target_os = "macos"))]
            let parsed_jvm_args: Vec<String> = parsed_jvm_args.into_iter().filter(|s| !s.contains("-XstartOnFirstThread")).collect();
            
            final_args.extend(parsed_jvm_args);
        } else {
            // Some versions have `arguments` but no `jvm` object, so we must manually add the classpath
            final_args.push("-cp".to_string());
            final_args.push(classpath.clone());
        }
        
        final_args.push(meta.main_class.clone());

        if let Some(game_args) = &args.game {
            final_args.extend(core::launcher::parse_arguments(game_args, &placeholders, &active_features));
        }
    } else {
        // Fallback for old versions (1.12.2 and older)
        final_args.push("-cp".to_string());
        final_args.push(classpath.clone());
        
        final_args.push(meta.main_class.clone());
        if let Some(mc_args) = &meta.minecraft_arguments {
            let parsed_old = mc_args.split_whitespace().map(|s| core::launcher::replace_placeholders(s, &placeholders)).collect::<Vec<String>>();
            final_args.extend(parsed_old);
        }
    }

    let working_dir = game_dir.to_string_lossy().to_string();

    // 8. Spawn process
    let _ = app.emit("mc-progress", core::downloader::ProgressPayload {
        instance_id: instance_id.clone(),
        task: "启动进程...".to_string(),
        progress: -1.0,
        text: "拉起 Java".to_string(),
    });
    core::process_manager::spawn_minecraft(app, &resolved_java_path, final_args, &working_dir)
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
fn delete_account(uuid: String) -> Result<(), String> {
    let mut accounts = core::config::load_accounts().unwrap_or_else(|_| vec![]);
    accounts.retain(|a| a.uuid != uuid);
    core::config::save_accounts(&accounts).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_accounts() -> Result<Vec<Account>, String> {
    core::config::load_accounts().map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_instance(name: String, mc_version: String, loader: LoaderType, use_performance_preset: bool) -> Result<Instance, String> {
    let mut instance = core::instance::create_instance(name, mc_version.clone(), loader.clone()).map_err(|e| e.to_string())?;

    // If using Fabric, fetch its meta automatically
    if let LoaderType::Fabric = loader {
        if let Ok(loader_version) = core::fabric_manager::fetch_latest_fabric_loader(&mc_version).await {
            let _ = core::fabric_manager::fetch_fabric_meta(&mc_version, &loader_version).await;
            
            // Update instance with loader version
            instance.loader_version = Some(loader_version);
            let _ = core::instance::save_instance(&instance);
            
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
fn get_settings() -> Result<AppSettings, String> {
    core::config::load_settings().map_err(|e| e.to_string())
}

#[tauri::command]
fn save_settings(settings: AppSettings) -> Result<(), String> {
    core::config::save_settings(&settings).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_minecraft_versions() -> Result<crate::core::minecraft::VersionManifest, String> {
    core::minecraft::fetch_version_manifest().await.map_err(|e| e.to_string())
}

use window_vibrancy::apply_mica;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
      add_offline_account,
      delete_account,
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
      install_modpack,
      get_local_mods,
      toggle_mod,
      delete_mod,
      open_instance_folder,
      export_instance,
      get_java_download_url,
      scan_java_installations,
      get_settings,
      save_settings,
      start_msa_login,
      poll_msa_token
    ])
    .setup(|app| {
            app.handle().plugin(tauri_plugin_dialog::init())?;
            app.handle().plugin(tauri_plugin_fs::init())?;
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            let window = app.get_webview_window("main").unwrap();

            #[cfg(target_os = "windows")]
            {
                // 尝试应用 Mica（仅 Win11）
                if apply_mica(&window, Some(true)).is_err() {
                    // 如果系统不支持 Mica (比如 Win10)，则必须关闭 body 的透明背景，否则会出现白屏
                    // 这里我们通过执行前端 JS 来动态移除 body 的 transparent 样式，恢复默认的深色背景
                    let _ = window.eval("document.body.style.backgroundColor = '#202020'; document.documentElement.style.backgroundColor = '#202020';");
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
