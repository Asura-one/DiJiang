/// Parse and display changelog entries between two versions.
const CHANGELOG: &str = include_str!("changelog.md");

/// Extract changelog entries between (exclusive of `from_version`) and (inclusive of `to_version`).
/// Versions should be semver strings like "0.1.0", "0.1.2".
/// Extract changelog entries between (exclusive of `from_version`) and (inclusive of `to_version`).
/// Versions should be semver strings like "0.1.0", "0.1.2".
/// Extract changelog entries between (exclusive of `from_version`) and (inclusive of `to_version`).
/// Versions should be semver strings like "0.1.0", "0.1.2".
pub fn changelog_between(from_version: &str, to_version: &str) -> String {
    if from_version == to_version {
        return String::new();
    }

    let mut entries = Vec::new();
    let mut in_range = false;
    let mut current_version = String::new();
    let mut current_entry = Vec::new();
    let mut broke = false;

    for line in CHANGELOG.lines() {
        if let Some(version) = line.strip_prefix("## ") {
            let version = version.split_whitespace().next().unwrap_or(version);

            if in_range {
                // Save previous entry
                if !current_entry.is_empty() {
                    entries.push((current_version.clone(), current_entry.clone()));
                }

                // Check if we hit the from_version boundary AFTER saving the previous entry
                // from_version is exclusive, so we stop when we reach it
                if version == from_version {
                    broke = true;
                    break;
                }
            }

            if version == to_version {
                in_range = true;
            }

            current_version = version.to_string();
            current_entry = vec![line.to_string()];
        } else if in_range {
            current_entry.push(line.to_string());
        }
    }

    // Save last entry only if we reached end of file (not after a break)
    if in_range && !broke && !current_entry.is_empty() {
        entries.push((current_version.clone(), current_entry.clone()));
    }

    if entries.is_empty() {
        return String::new();
    }

    let mut output = String::new();
    for (_version, lines) in entries.iter().rev() {
        let entry = lines.join("\n").trim().to_string();
        if !entry.is_empty() {
            if !output.is_empty() {
                output.push_str("\n\n");
            }
            output.push_str(&entry);
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_changelog_between_versions() {
        let result = changelog_between("0.1.0", "0.1.2");
        assert!(result.contains("0.1.2"), "should contain 0.1.2: {result}");
        assert!(result.contains("0.1.1"), "should contain 0.1.1: {result}");
        assert!(
            !result.contains("## 0.1.0"),
            "should NOT contain 0.1.0: {result}"
        );
    }

    #[test]
    fn test_changelog_same_version() {
        let result = changelog_between("0.1.2", "0.1.2");
        assert!(result.is_empty(), "should be empty: {result}");
    }

    #[test]
    fn test_changelog_single_version() {
        let result = changelog_between("0.1.1", "0.1.2");
        assert!(result.contains("0.1.2"), "should contain 0.1.2: {result}");
        assert!(
            !result.contains("0.1.1"),
            "should NOT contain 0.1.1: {result}"
        );
    }
}
