use crate::models::instance::{Instance, LoaderType};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

fn ensure_dir(path: PathBuf) -> PathBuf {
    if !path.exists() {
        fs::create_dir_all(&path).unwrap();
    }
    path
}

pub fn get_instances_dir() -> PathBuf {
    let mut path = crate::core::config::get_app_dir();
    path.push("instances");
    ensure_dir(path)
}

pub fn get_instance_dir(instance_id: &str) -> PathBuf {
    let mut path = get_instances_dir();
    path.push(instance_id);
    path
}

pub fn get_instance_game_dir(instance_id: &str) -> PathBuf {
    let mut path = get_instance_dir(instance_id);
    path.push("game");
    ensure_dir(path)
}

fn get_instance_game_subdir(instance_id: &str, name: &str) -> PathBuf {
    let mut path = get_instance_game_dir(instance_id);
    path.push(name);
    ensure_dir(path)
}

pub fn get_instance_mods_dir(instance_id: &str) -> PathBuf {
    get_instance_game_subdir(instance_id, "mods")
}

pub fn get_instance_resourcepacks_dir(instance_id: &str) -> PathBuf {
    get_instance_game_subdir(instance_id, "resourcepacks")
}

pub fn get_instance_shaderpacks_dir(instance_id: &str) -> PathBuf {
    get_instance_game_subdir(instance_id, "shaderpacks")
}

pub fn get_instance_saves_dir(instance_id: &str) -> PathBuf {
    get_instance_game_subdir(instance_id, "saves")
}

pub fn get_instance_logs_dir(instance_id: &str) -> PathBuf {
    get_instance_game_subdir(instance_id, "logs")
}

pub fn get_instance_config_dir(instance_id: &str) -> PathBuf {
    get_instance_game_subdir(instance_id, "config")
}

pub fn get_instance_crash_reports_dir(instance_id: &str) -> PathBuf {
    get_instance_game_subdir(instance_id, "crash-reports")
}

pub fn get_instance_work_dir(instance_id: &str) -> PathBuf {
    let mut path = get_instance_dir(instance_id);
    path.push("work");
    ensure_dir(path)
}

pub fn get_instance_natives_dir(instance_id: &str) -> PathBuf {
    let mut path = get_instance_work_dir(instance_id);
    path.push("natives");
    ensure_dir(path)
}

pub fn save_instance(instance: &Instance) -> Result<(), anyhow::Error> {
    let instance_dir = get_instance_dir(&instance.id);
    let config_path = instance_dir.join("instance.json");
    let json = serde_json::to_string_pretty(instance)?;
    fs::write(config_path, json)?;
    Ok(())
}

pub fn create_instance(
    name: String,
    mc_version: String,
    loader: LoaderType,
) -> Result<Instance, anyhow::Error> {
    let id = Uuid::new_v4().to_string();
    let instance = Instance {
        id: id.clone(),
        name,
        mc_version,
        loader,
        loader_version: None,
        java_path: None,
        memory_min: Some(1024),
        memory_max: Some(4096),
    };

    let instance_dir = get_instance_dir(&id);
    fs::create_dir_all(&instance_dir)?;

    let config_path = instance_dir.join("instance.json");
    let json = serde_json::to_string_pretty(&instance)?;
    fs::write(config_path, json)?;

    // create game dir
    get_instance_game_dir(&id);
    get_instance_mods_dir(&id);
    get_instance_resourcepacks_dir(&id);
    get_instance_shaderpacks_dir(&id);
    get_instance_saves_dir(&id);
    get_instance_logs_dir(&id);
    get_instance_config_dir(&id);
    get_instance_crash_reports_dir(&id);
    get_instance_work_dir(&id);

    Ok(instance)
}

pub fn get_instance_by_id(id: &str) -> Result<Instance, anyhow::Error> {
    let instances = load_instances()?;
    instances.into_iter().find(|i| i.id == id)
        .ok_or_else(|| anyhow::anyhow!("Instance not found"))
}

pub fn load_instances() -> Result<Vec<Instance>, anyhow::Error> {
    let mut instances = Vec::new();
    let dir = get_instances_dir();

    if dir.exists() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let config_path = path.join("instance.json");
                if config_path.exists() {
                    let data = fs::read_to_string(config_path)?;
                    if let Ok(instance) = serde_json::from_str(&data) {
                        instances.push(instance);
                    }
                }
            }
        }
    }

    Ok(instances)
}

pub fn delete_instance(id: &str) -> Result<(), anyhow::Error> {
    let mut dir = get_instances_dir();
    dir.push(id);
    if dir.exists() {
        fs::remove_dir_all(dir)?;
    }
    Ok(())
}
