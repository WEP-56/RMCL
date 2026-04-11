use crate::core::paths;
use crate::core::placeholders::replace_in_text;
use crate::core::rules::evaluate_rules;
use crate::models::manifest::{Argument, ArgumentValue, VersionMeta};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchFeatureOptions {
    pub is_demo_user: bool,
    pub has_custom_resolution: bool,
    pub has_quick_plays_support: bool,
    pub is_quick_play_singleplayer: bool,
    pub is_quick_play_multiplayer: bool,
    pub is_quick_play_realms: bool,
}

impl LaunchFeatureOptions {
    pub fn to_rule_map(&self) -> HashMap<String, bool> {
        HashMap::from([
            ("is_demo_user".to_string(), self.is_demo_user),
            (
                "has_custom_resolution".to_string(),
                self.has_custom_resolution,
            ),
            (
                "has_quick_plays_support".to_string(),
                self.has_quick_plays_support,
            ),
            (
                "is_quick_play_singleplayer".to_string(),
                self.is_quick_play_singleplayer,
            ),
            (
                "is_quick_play_multiplayer".to_string(),
                self.is_quick_play_multiplayer,
            ),
            (
                "is_quick_play_realms".to_string(),
                self.is_quick_play_realms,
            ),
        ])
    }
}

pub fn build_classpath(meta: &VersionMeta) -> String {
    build_classpath_with_features(meta, &default_launch_features())
}

pub fn build_classpath_with_features(
    meta: &VersionMeta,
    active_features: &HashMap<String, bool>,
) -> String {
    let lib_dir = paths::get_libraries_dir();
    let mut cp = Vec::new();

    for lib in &meta.libraries {
        // Evaluate rules if they exist
        if let Some(rules) = &lib.rules {
            if !evaluate_rules(rules, &active_features) {
                continue;
            }
        }

        if let Some(downloads) = &lib.downloads {
            if let Some(artifact) = &downloads.artifact {
                let path = lib_dir.join(&artifact.path);
                cp.push(path.to_string_lossy().to_string());
            }
        }
    }

    // Add client jar
    let mut client_jar = paths::get_versions_dir();
    client_jar.push(&meta.id);
    client_jar.push(format!("{}.jar", meta.id));
    cp.push(client_jar.to_string_lossy().to_string());

    // Join with OS specific separator
    #[cfg(target_os = "windows")]
    let separator = ";";
    #[cfg(not(target_os = "windows"))]
    let separator = ":";

    // Deduplicate classpath entries
    let mut unique_cp = Vec::new();
    for p in cp {
        if !unique_cp.contains(&p) {
            unique_cp.push(p);
        }
    }

    // On Windows, the classpath might get extremely long and exceed the maximum command line length limit (8191 chars)
    // To solve this properly, we could write the classpath to a temporary args file and pass @args.txt to java.
    // For now, this implementation should work for most modpacks.
    unique_cp.join(separator)
}

pub fn parse_arguments(
    args: &[Argument],
    placeholders: &HashMap<String, String>,
    features: &HashMap<String, bool>,
) -> Vec<String> {
    let mut result = Vec::new();

    for arg in args {
        match arg {
            Argument::String(s) => {
                result.push(replace_in_text(s, placeholders));
            }
            Argument::Rule { rules, value } => {
                if evaluate_rules(rules, features) {
                    match value {
                        ArgumentValue::Single(s) => {
                            result.push(replace_in_text(s, placeholders));
                        }
                        ArgumentValue::Multiple(vec) => {
                            for s in vec {
                                result.push(replace_in_text(s, placeholders));
                            }
                        }
                    }
                }
            }
        }
    }

    result
}

pub fn default_launch_features() -> HashMap<String, bool> {
    LaunchFeatureOptions::default().to_rule_map()
}

pub fn resolve_logging_argument(
    meta: &VersionMeta,
    logging_config_path: Option<&Path>,
) -> Option<String> {
    let logging_argument = meta
        .logging
        .as_ref()
        .and_then(|logging| logging.client.as_ref())
        .and_then(|client| client.argument.as_ref())?;

    Some(if let Some(path) = logging_config_path {
        logging_argument.replace("${path}", path.to_string_lossy().as_ref())
    } else {
        logging_argument.clone()
    })
}

pub fn build_launch_command_args(
    meta: &VersionMeta,
    placeholders: &HashMap<String, String>,
    classpath: &str,
    natives_dir: &Path,
    max_memory: u32,
    feature_options: &LaunchFeatureOptions,
    logging_config_path: Option<&Path>,
) -> Vec<String> {
    let mut final_args = Vec::new();
    final_args.push(format!("-Xmx{}M", max_memory));

    #[cfg(target_os = "macos")]
    final_args.push("-XstartOnFirstThread".to_string());

    final_args.push("-XX:+UnlockExperimentalVMOptions".to_string());
    final_args.push("-XX:+UseG1GC".to_string());
    final_args.push("-XX:G1NewSizePercent=20".to_string());
    final_args.push("-XX:G1ReservePercent=20".to_string());
    final_args.push("-XX:MaxGCPauseMillis=50".to_string());
    final_args.push("-XX:G1HeapRegionSize=32M".to_string());
    final_args.push(format!(
        "-Djava.library.path={}",
        natives_dir.to_string_lossy()
    ));

    if let Some(logging_argument) = resolve_logging_argument(meta, logging_config_path) {
        final_args.push(logging_argument);
    }

    let active_features = feature_options.to_rule_map();

    if let Some(arguments) = &meta.arguments {
        if let Some(jvm_args) = &arguments.jvm {
            let parsed_jvm_args = parse_arguments(jvm_args, placeholders, &active_features);

            #[cfg(not(target_os = "macos"))]
            let parsed_jvm_args: Vec<String> = parsed_jvm_args
                .into_iter()
                .filter(|arg| !arg.contains("-XstartOnFirstThread"))
                .collect();

            final_args.extend(parsed_jvm_args);
        } else {
            final_args.push("-cp".to_string());
            final_args.push(classpath.to_string());
        }

        final_args.push(meta.main_class.clone());
        if let Some(game_args) = &arguments.game {
            final_args.extend(parse_arguments(game_args, placeholders, &active_features));
        }
    } else {
        final_args.push("-cp".to_string());
        final_args.push(classpath.to_string());
        final_args.push(meta.main_class.clone());
        if let Some(minecraft_arguments) = &meta.minecraft_arguments {
            final_args.extend(
                minecraft_arguments
                    .split_whitespace()
                    .map(|arg| replace_in_text(arg, placeholders)),
            );
        }
    }

    final_args
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::manifest::{
        Arguments, LoggingClientConfig, LoggingConfig, LoggingFile, Rule, VersionMeta,
    };

    fn test_meta(arguments: Option<Arguments>, minecraft_arguments: Option<&str>) -> VersionMeta {
        VersionMeta {
            id: "1.20.1".to_string(),
            version_type: "release".to_string(),
            main_class: "net.minecraft.client.main.Main".to_string(),
            inherits_from: None,
            minecraft_arguments: minecraft_arguments.map(str::to_string),
            arguments,
            asset_index: None,
            assets: None,
            downloads: None,
            libraries: vec![],
            java_version: None,
            logging: None,
        }
    }

    #[test]
    fn build_launch_command_args_respects_default_features() {
        let args = Arguments {
            jvm: Some(vec![
                Argument::String("-cp".to_string()),
                Argument::String("${classpath}".to_string()),
                Argument::String("-Ddemo=true".to_string()),
                Argument::Rule {
                    rules: vec![Rule {
                        action: "allow".to_string(),
                        os: None,
                        features: Some(HashMap::from([(
                            "has_custom_resolution".to_string(),
                            true,
                        )])),
                    }],
                    value: ArgumentValue::Single("-Dshould-not-exist=true".to_string()),
                },
            ]),
            game: Some(vec![
                Argument::String("--username".to_string()),
                Argument::String("${auth_player_name}".to_string()),
            ]),
        };
        let meta = test_meta(Some(args), None);
        let placeholders = HashMap::from([
            ("classpath".to_string(), "a.jar;b.jar".to_string()),
            ("auth_player_name".to_string(), "Steve".to_string()),
        ]);

        let final_args = build_launch_command_args(
            &meta,
            &placeholders,
            "a.jar;b.jar",
            Path::new("C:/tmp/natives"),
            2048,
            &LaunchFeatureOptions::default(),
            None,
        );

        assert!(final_args.iter().any(|arg| arg == "-Xmx2048M"));
        assert!(final_args.iter().any(|arg| arg == "-Ddemo=true"));
        assert!(final_args.iter().any(|arg| arg == "a.jar;b.jar"));
        assert!(final_args
            .iter()
            .any(|arg| arg == "net.minecraft.client.main.Main"));
        assert!(final_args.iter().any(|arg| arg == "Steve"));
        assert!(!final_args
            .iter()
            .any(|arg| arg == "-Dshould-not-exist=true"));
    }

    #[test]
    fn build_launch_command_args_supports_legacy_arguments_and_logging() {
        let mut meta = test_meta(None, Some("--demo --username ${auth_player_name}"));
        meta.logging = Some(LoggingConfig {
            client: Some(LoggingClientConfig {
                argument: Some("-Dlog4j.configurationFile=${path}".to_string()),
                file: Some(LoggingFile {
                    id: "client-1.20.xml".to_string(),
                    sha1: "sha1".to_string(),
                    size: 1,
                    url: "https://example.com/client-1.20.xml".to_string(),
                }),
            }),
        });

        let placeholders = HashMap::from([("auth_player_name".to_string(), "Alex".to_string())]);
        let logging_path = Path::new("C:/tmp/client-1.20.xml");
        let final_args = build_launch_command_args(
            &meta,
            &placeholders,
            "cp.jar",
            Path::new("C:/tmp/natives"),
            3072,
            &LaunchFeatureOptions::default(),
            Some(logging_path),
        );

        assert!(final_args.iter().any(|arg| arg == "-cp"));
        assert!(final_args.iter().any(|arg| arg == "cp.jar"));
        assert!(final_args
            .iter()
            .any(|arg| arg == "-Dlog4j.configurationFile=C:/tmp/client-1.20.xml"));
        assert!(final_args.iter().any(|arg| arg == "--demo"));
        assert!(final_args.iter().any(|arg| arg == "Alex"));
    }
}
