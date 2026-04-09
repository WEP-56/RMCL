use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    pub id: String,
    pub name: String,
    pub mc_version: String,
    pub loader: LoaderType,
    pub loader_version: Option<String>,
    pub java_path: Option<String>,
    pub memory_min: Option<u32>,
    pub memory_max: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoaderType {
    Vanilla,
    Fabric,
    Forge,
    Quilt,
    NeoForge,
}
