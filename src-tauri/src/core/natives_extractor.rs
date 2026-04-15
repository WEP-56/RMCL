use crate::core::instance;
use crate::models::manifest::VersionMeta;
use std::fs;
use std::path::PathBuf;

pub fn extract_natives(instance_id: &str, meta: &VersionMeta) -> Result<PathBuf, anyhow::Error> {
    let natives_dir = instance::get_instance_natives_dir(instance_id);

    // Clean up old natives dir if it exists
    if natives_dir.exists() {
        fs::remove_dir_all(&natives_dir)?;
    }
    fs::create_dir_all(&natives_dir)?;

    let lib_dir = crate::core::paths::get_libraries_dir();

    for lib in &meta.libraries {
        if let Some(natives) = &lib.natives {
            #[cfg(target_os = "windows")]
            let os_key = "windows";
            #[cfg(target_os = "macos")]
            let os_key = "osx";
            #[cfg(target_os = "linux")]
            let os_key = "linux";

            if let Some(native_classifier) = natives.get(os_key) {
                #[cfg(target_arch = "x86_64")]
                let arch_str = "64";
                #[cfg(target_arch = "x86")]
                let arch_str = "32";
                #[cfg(target_arch = "aarch64")]
                let arch_str = "arm64";

                let actual_classifier = native_classifier.replace("${arch}", arch_str);

                if let Some(downloads) = &lib.downloads {
                    if let Some(classifiers) = &downloads.classifiers {
                        if let Some(native_artifact) = classifiers.get(&actual_classifier) {
                            let jar_path = lib_dir.join(&native_artifact.path);
                            
                            if jar_path.exists() {
                                // Extract jar file (zip)
                                let file = fs::File::open(&jar_path)?;
                                let mut archive = zip::ZipArchive::new(file)?;
                                
                                for i in 0..archive.len() {
                                    let mut file = archive.by_index(i)?;
                                    let outpath = match file.enclosed_name() {
                                        Some(path) => path.to_owned(),
                                        None => continue,
                                    };

                                    // Check exclude rules (e.g. META-INF)
                                    let mut exclude = false;
                                    if let Some(extract_rules) = &lib.extract {
                                        if let Some(excludes) = &extract_rules.exclude {
                                            for ex in excludes {
                                                if outpath.to_string_lossy().starts_with(ex) {
                                                    exclude = true;
                                                    break;
                                                }
                                            }
                                        }
                                    }

                                    if exclude { continue; }

                                    let target_path = natives_dir.join(&outpath);
                                    if file.is_dir() {
                                        fs::create_dir_all(&target_path)?;
                                    } else {
                                        if let Some(p) = target_path.parent() {
                                            fs::create_dir_all(p)?;
                                        }
                                        let mut outfile = fs::File::create(&target_path)?;
                                        std::io::copy(&mut file, &mut outfile)?;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(natives_dir)
}
