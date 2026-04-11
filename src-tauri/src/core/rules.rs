use crate::models::manifest::{OsRule, Rule};
use std::collections::HashMap;

pub fn evaluate_rules(rules: &[Rule], active_features: &HashMap<String, bool>) -> bool {
    if rules.is_empty() {
        return true;
    }

    // Mojang rule semantics: start from denied, then apply matching rules in order.
    let mut allowed = false;
    for rule in rules {
        if matches_rule(rule, active_features) {
            allowed = rule.action == "allow";
        }
    }

    allowed
}

fn matches_rule(rule: &Rule, active_features: &HashMap<String, bool>) -> bool {
    matches_os(rule.os.as_ref()) && matches_features(rule.features.as_ref(), active_features)
}

fn matches_os(os_rule: Option<&OsRule>) -> bool {
    let Some(os_rule) = os_rule else {
        return true;
    };

    let name_match = os_rule.name.as_deref().map(matches_os_name).unwrap_or(true);
    let arch_match = os_rule.arch.as_deref().map(matches_os_arch).unwrap_or(true);

    // `os.version` may contain regex in official manifests. Step 1 only needs
    // `os.name` / `os.arch` / `features`, so we intentionally skip version here.
    name_match && arch_match
}

fn matches_os_name(required: &str) -> bool {
    let normalized = required.to_ascii_lowercase();
    match normalized.as_str() {
        "windows" => cfg!(target_os = "windows"),
        "osx" | "macos" => cfg!(target_os = "macos"),
        "linux" => cfg!(target_os = "linux"),
        _ => normalized == current_os_name(),
    }
}

fn matches_os_arch(required: &str) -> bool {
    let normalized = required.to_ascii_lowercase();

    #[cfg(target_arch = "x86_64")]
    {
        matches!(normalized.as_str(), "x86_64" | "amd64")
    }
    #[cfg(target_arch = "x86")]
    {
        matches!(
            normalized.as_str(),
            "x86" | "i386" | "i686" | "x86_32" | "32"
        )
    }
    #[cfg(target_arch = "aarch64")]
    {
        matches!(normalized.as_str(), "aarch64" | "arm64")
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86", target_arch = "aarch64")))]
    {
        normalized == std::env::consts::ARCH.to_ascii_lowercase()
    }
}

fn matches_features(
    required_features: Option<&HashMap<String, bool>>,
    active_features: &HashMap<String, bool>,
) -> bool {
    let Some(required_features) = required_features else {
        return true;
    };

    required_features
        .iter()
        .all(|(feature_name, required_value)| {
            active_features.get(feature_name).copied().unwrap_or(false) == *required_value
        })
}

fn current_os_name() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        return "windows";
    }
    #[cfg(target_os = "macos")]
    {
        return "osx";
    }
    #[cfg(target_os = "linux")]
    {
        return "linux";
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        return "unknown";
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::manifest::{OsRule, Rule};

    fn make_rule(action: &str, os_name: Option<&str>, feature: Option<(&str, bool)>) -> Rule {
        let features = feature.map(|(name, value)| {
            let mut map = HashMap::new();
            map.insert(name.to_string(), value);
            map
        });

        Rule {
            action: action.to_string(),
            os: os_name.map(|name| OsRule {
                name: Some(name.to_string()),
                version: None,
                arch: None,
            }),
            features,
        }
    }

    #[test]
    fn empty_rules_allow_by_default() {
        assert!(evaluate_rules(&[], &HashMap::new()));
    }

    #[test]
    fn unmatched_allow_rule_stays_disallowed() {
        let rules = vec![make_rule("allow", Some("definitely-not-an-os"), None)];
        let mut features = HashMap::new();
        features.insert("unused".to_string(), true);
        assert!(!evaluate_rules(&rules, &features));
    }

    #[test]
    fn matching_allow_then_disallow_uses_last_match() {
        let os_rule_name = current_os_name();
        let rules = vec![
            make_rule("allow", None, None),
            make_rule("disallow", Some(os_rule_name), None),
        ];

        assert!(!evaluate_rules(&rules, &HashMap::new()));
    }

    #[test]
    fn feature_defaults_to_false_when_missing() {
        let rules = vec![make_rule(
            "allow",
            None,
            Some(("has_custom_resolution", true)),
        )];
        assert!(!evaluate_rules(&rules, &HashMap::new()));
    }

    #[test]
    fn feature_rule_allows_when_explicitly_enabled() {
        let rules = vec![make_rule("allow", None, Some(("is_demo_user", true)))];
        let mut active_features = HashMap::new();
        active_features.insert("is_demo_user".to_string(), true);

        assert!(evaluate_rules(&rules, &active_features));
    }
}
