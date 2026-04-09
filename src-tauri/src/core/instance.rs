use crate::models::instance::{Instance, LoaderType};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

pub fn get_instances_dir() -> PathBuf {
    let mut path = crate::core::config::get_app_dir();
    path.push("instances");
    if !path.exists() {
        fs::create_dir_all(&path).unwrap();
    }
    path
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

    let mut instance_dir = get_instances_dir();
    instance_dir.push(&id);
    fs::create_dir_all(&instance_dir)?;

    let config_path = instance_dir.join("instance.json");
    let json = serde_json::to_string_pretty(&instance)?;
    fs::write(config_path, json)?;

    Ok(instance)
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
