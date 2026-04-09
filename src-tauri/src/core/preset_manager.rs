use crate::core::modrinth_api::get_project_versions;
use crate::core::mod_manager::install_mod_version;

pub async fn install_performance_preset(
    instance_id: &str,
    game_version: &str,
) -> Result<(), anyhow::Error> {
    // Commonly used performance mods on Modrinth
    let target_mods = vec!["sodium", "lithium", "iris"];

    for mod_slug in target_mods {
        let versions = get_project_versions(
            mod_slug,
            Some(vec!["fabric"]),
            Some(vec![game_version]),
        )
        .await?;

        if !versions.is_empty() {
            // Install the latest compatible version
            install_mod_version(instance_id, &versions[0]).await?;
        }
    }

    Ok(())
}
