use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResult {
    pub hits: Vec<Project>,
    pub offset: u32,
    pub limit: u32,
    pub total_hits: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub project_id: String,
    pub project_type: String,
    pub slug: String,
    pub author: String,
    pub title: String,
    pub description: String,
    pub categories: Vec<String>,
    pub display_categories: Vec<String>,
    pub versions: Vec<String>,
    pub downloads: u32,
    pub follows: u32,
    pub icon_url: Option<String>,
    pub date_created: String,
    pub date_modified: String,
    pub latest_version: String,
    pub license: String,
    pub client_side: String,
    pub server_side: String,
    pub gallery: Vec<String>,
    pub featured_gallery: Option<String>,
    pub color: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Version {
    pub name: String,
    pub version_number: String,
    pub changelog: Option<String>,
    pub dependencies: Option<Vec<Dependency>>,
    pub game_versions: Vec<String>,
    pub version_type: String,
    pub loaders: Vec<String>,
    pub featured: bool,
    pub id: String,
    pub project_id: String,
    pub author_id: String,
    pub date_published: String,
    pub downloads: u32,
    pub files: Vec<File>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Dependency {
    pub version_id: Option<String>,
    pub project_id: Option<String>,
    pub file_name: Option<String>,
    pub dependency_type: String, // required, optional, incompatible, embedded
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct File {
    pub hashes: Hashes,
    pub url: String,
    pub filename: String,
    pub primary: bool,
    pub size: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Hashes {
    pub sha512: String,
    pub sha1: String,
}
