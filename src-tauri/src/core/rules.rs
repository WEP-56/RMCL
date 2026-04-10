use crate::models::manifest::Rule;
use std::collections::HashMap;

pub fn evaluate_rules(rules: &[Rule], active_features: &HashMap<String, bool>) -> bool {
    if rules.is_empty() {
        return true; // Default allow if no rules
    }

    let mut result = false;
    let mut has_matched_rule = false;

    for rule in rules {
        let mut match_os = true;
        let mut match_features = true;
        let mut has_conditions = false;

        if let Some(os) = &rule.os {
            has_conditions = true;
            if let Some(name) = &os.name {
                #[cfg(target_os = "windows")]
                { match_os &= name == "windows"; }
                #[cfg(target_os = "macos")]
                { match_os &= name == "osx"; }
                #[cfg(target_os = "linux")]
                { match_os &= name == "linux"; }
            }
            if let Some(arch) = &os.arch {
                #[cfg(target_arch = "x86")]
                { match_os &= arch == "x86"; }
                #[cfg(target_arch = "x86_64")]
                { match_os &= arch == "x86_64" || arch == "amd64"; }
                #[cfg(target_arch = "aarch64")]
                { match_os &= arch == "aarch64" || arch == "arm64"; }
            }
            // version string regex matching is ignored for simplicity for now
        }

        if let Some(req_features) = &rule.features {
            has_conditions = true;
            for (feature_name, required_val) in req_features {
                let actual_val = active_features.get(feature_name).copied().unwrap_or(false);
                if actual_val != *required_val {
                    match_features = false;
                    break;
                }
            }
        }

        // If rule has no conditions (no `os`, no `features`), it's a global rule
        if !has_conditions {
            has_matched_rule = true;
            if rule.action == "allow" {
                result = true;
            } else if rule.action == "disallow" {
                result = false;
            }
        } else if match_os && match_features {
            has_matched_rule = true;
            if rule.action == "allow" {
                result = true;
            } else if rule.action == "disallow" {
                result = false;
            }
        }
    }

    if !has_matched_rule {
        // Default behavior: if first rule is allow, then default is disallow
        // If first rule is disallow, then default is allow
        if let Some(first_rule) = rules.first() {
            if first_rule.action == "allow" {
                return false;
            } else {
                return true;
            }
        }
    }

    result
}
