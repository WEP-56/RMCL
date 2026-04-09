use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub java_path: String,
    pub max_memory: u32,
    pub game_directory: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            java_path: "java".to_string(),
            max_memory: 2048,
            game_directory: None,
        }
    }
}