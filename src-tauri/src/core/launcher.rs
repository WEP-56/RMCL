use crate::core::paths;
use crate::models::account::Account;
use crate::models::manifest::{Argument, ArgumentValue, VersionMeta};
use std::collections::HashMap;

pub struct LaunchOptions {
    pub account: Account,
    pub instance_dir: String,
    pub min_memory: u32,
    pub max_memory: u32,
    pub java_path: String,
}

pub fn build_classpath(meta: &VersionMeta) -> String {
    let lib_dir = paths::get_libraries_dir();
    let mut cp = Vec::new();

    for lib in &meta.libraries {
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

pub fn replace_placeholders(arg: &str, placeholders: &HashMap<&str, String>) -> String {
    let mut result = arg.to_string();
    for (key, value) in placeholders {
        let target = format!("${{{}}}", key);
        result = result.replace(&target, value);
    }
    result
}

pub fn parse_arguments(
    args: &Vec<Argument>,
    placeholders: &HashMap<&str, String>,
) -> Vec<String> {
    let mut result = Vec::new();

    for arg in args {
        match arg {
            Argument::String(s) => {
                result.push(replace_placeholders(s, placeholders));
            }
            Argument::Rule { rules: _rules, value } => {
                // TODO: Full rule evaluation. Assuming allowed for now.
                let allowed = true; 
                
                if allowed {
                    match value {
                        ArgumentValue::Single(s) => {
                            result.push(replace_placeholders(s, placeholders));
                        }
                        ArgumentValue::Multiple(vec) => {
                            for s in vec {
                                result.push(replace_placeholders(s, placeholders));
                            }
                        }
                    }
                }
            }
        }
    }

    result
}
