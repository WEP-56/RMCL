use crate::models::modrinth::{SearchResult, Version};
use reqwest::Client;

const MODRINTH_API_URL: &str = "https://api.modrinth.com/v2";

pub async fn search_projects(
    query: &str,
    loaders: Option<Vec<&str>>,
    game_versions: Option<Vec<&str>>,
    limit: u32,
    offset: u32,
) -> Result<SearchResult, anyhow::Error> {
    let client = Client::new();
    
    let mut facets = Vec::new();
    
    if let Some(l) = loaders {
        let loader_facets: Vec<String> = l.iter().map(|loader| format!("categories:{}", loader)).collect();
        facets.push(loader_facets);
    }
    
    if let Some(gv) = game_versions {
        let version_facets: Vec<String> = gv.iter().map(|v| format!("versions:{}", v)).collect();
        facets.push(version_facets);
    }

    let limit_str = limit.to_string();
    let offset_str = offset.to_string();
    
    let mut query_params = vec![
        ("query", query.to_string()),
        ("limit", limit_str),
        ("offset", offset_str),
    ];

    if !facets.is_empty() {
        let facets_json = serde_json::to_string(&facets)?;
        query_params.push(("facets", facets_json));
    }

    let request = client.get(&format!("{}/search", MODRINTH_API_URL))
        .query(&query_params)
        .header("User-Agent", "RustMCLauncher/0.1.0");

    let response = request.send().await?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("Modrinth API error: {}", response.status()));
    }

    let result: SearchResult = response.json().await?;
    Ok(result)
}

pub async fn get_project_versions(
    project_id: &str,
    loaders: Option<Vec<&str>>,
    game_versions: Option<Vec<&str>>,
) -> Result<Vec<Version>, anyhow::Error> {
    let client = Client::new();
    
    let mut query_params = Vec::new();

    if let Some(l) = loaders {
        let json = serde_json::to_string(&l)?;
        query_params.push(("loaders", json));
    }

    if let Some(gv) = game_versions {
        let json = serde_json::to_string(&gv)?;
        query_params.push(("game_versions", json));
    }

    let request = client.get(&format!("{}/project/{}/version", MODRINTH_API_URL, project_id))
        .query(&query_params)
        .header("User-Agent", "RustMCLauncher/0.1.0");

    let response = request.send().await?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("Modrinth API error: {}", response.status()));
    }

    let versions: Vec<Version> = response.json().await?;
    Ok(versions)
}
