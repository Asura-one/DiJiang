use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

/// A file entry within a template package.
#[derive(Debug, Clone, Deserialize)]
pub struct TemplateFileEntry {
    /// Relative path within the template (e.g. "spec/coding.md")
    pub path: String,
    /// Optional description of this file's purpose
    #[serde(default)]
    pub description: String,
    /// Whether to apply {{variable}} substitution (default: true)
    #[serde(default = "default_template_true")]
    pub template: bool,
}

fn default_template_true() -> bool {
    true
}

/// A variable that the template requires at render time.
#[derive(Debug, Clone, Deserialize)]
pub struct TemplateVariable {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub default: Option<String>,
}

/// Template manifest (TOML), defining a reusable template package.
#[derive(Debug, Clone, Deserialize)]
pub struct TemplateManifest {
    pub template: TemplateMeta,
    pub metadata: Option<TemplateMetadata>,
    #[serde(default)]
    pub files: Vec<TemplateFileEntry>,
    #[serde(default)]
    pub variables: Vec<TemplateVariable>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TemplateMeta {
    pub name: String,
    pub version: String,
    pub description: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TemplateMetadata {
    pub author: Option<String>,
    pub created: Option<String>,
    pub license: Option<String>,
}

/// A parsed remote template source.
#[derive(Debug, Clone)]
pub enum TemplateSource {
    /// gh:owner/repo[/path/to/template]
    GitHub {
        owner: String,
        repo: String,
        subpath: Option<String>,
        branch: String,
    },
    /// Raw HTTP(S) URL to a manifest file
    Url(String),
}

impl TemplateSource {
    /// Parse a source string into a TemplateSource.
    ///
    /// Supported formats:
    /// - `gh:owner/repo` — default branch, root of "templates/" dir
    /// - `gh:owner/repo/path` — subpath
    /// - `gh:owner/repo/path@branch` — specific branch
    /// - `https://raw.example.com/...` — raw URL
    pub fn parse(s: &str) -> Result<Self, String> {
        if let Some(rest) = s.strip_prefix("gh:") {
            let (path, branch) = if let Some(at_pos) = rest.find('@') {
                let (p, b) = rest.split_at(at_pos);
                (p, b[1..].to_string())
            } else {
                (rest, "main".to_string())
            };

            let parts: Vec<&str> = path.split('/').collect();
            if parts.len() < 2 {
                return Err(format!(
                    "Invalid gh: source '{}'. Expected gh:owner/repo[/path]",
                    s
                ));
            }
            let owner = parts[0].to_string();
            let repo = parts[1].to_string();
            let subpath = if parts.len() > 2 {
                Some(parts[2..].join("/"))
            } else {
                None
            };
            Ok(TemplateSource::GitHub {
                owner,
                repo,
                subpath,
                branch,
            })
        } else if s.starts_with("http://") || s.starts_with("https://") {
            Ok(TemplateSource::Url(s.to_string()))
        } else {
            Err(format!(
                "Unknown template source format '{}'. Use gh:owner/repo or a URL.",
                s
            ))
        }
    }

    /// Return a human-readable label for this source.
    pub fn label(&self) -> String {
        match self {
            TemplateSource::GitHub {
                owner,
                repo,
                subpath,
                branch,
            } => {
                let base = format!("gh:{}/{}@{}", owner, repo, branch);
                if let Some(sp) = subpath {
                    format!("{}/{}", base, sp)
                } else {
                    base
                }
            }
            TemplateSource::Url(url) => url.clone(),
        }
    }
}

/// A resolved, cached template package on disk.
#[derive(Debug, Clone)]
pub struct TemplatePackage {
    pub manifest: TemplateManifest,
    pub source: TemplateSource,
    pub root: PathBuf,
}

/// The template registry manages local template cache and remote fetching.
pub struct TemplateRegistry {
    /// Root directory for cached templates (~/.dijiang/templates/)
    cache_root: PathBuf,
}

impl Default for TemplateRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateRegistry {
    /// Create a new registry using the default cache directory.
    pub fn new() -> Self {
        let cache_root = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".dijiang")
            .join("templates");
        TemplateRegistry { cache_root }
    }

    /// Create a registry with a custom cache root (for testing).
    pub fn with_root(root: PathBuf) -> Self {
        TemplateRegistry { cache_root: root }
    }

    /// List all cached template packages.
    pub fn list_local(&self) -> Result<Vec<TemplatePackage>, String> {
        if !self.cache_root.exists() {
            return Ok(vec![]);
        }

        let mut packages = Vec::new();
        let entries = fs::read_dir(&self.cache_root)
            .map_err(|e| format!("Failed to read cache dir: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let manifest_path = path.join("manifest.toml");
            if !manifest_path.exists() {
                continue;
            }
            match self.load_manifest(&manifest_path) {
                Ok(manifest) => {
                    let source_label = Self::source_label_from_path(&path);
                    let source = TemplateSource::parse(&source_label).unwrap_or(
                        TemplateSource::Url("local".to_string()),
                    );
                    packages.push(TemplatePackage {
                        manifest,
                        source,
                        root: path,
                    });
                }
                Err(_) => continue,
            }
        }
        Ok(packages)
    }

    /// Pull a template from a remote source into the local cache.
    pub fn pull(&self, source: &TemplateSource) -> Result<TemplatePackage, String> {
        match source {
            TemplateSource::GitHub {
                owner,
                repo,
                subpath,
                branch,
            } => self.pull_github(owner, repo, subpath.as_deref(), branch),
            TemplateSource::Url(url) => self.pull_url(url),
        }
    }

    /// Pull from a GitHub repository.
    fn pull_github(
        &self,
        owner: &str,
        repo: &str,
        subpath: Option<&str>,
        branch: &str,
    ) -> Result<TemplatePackage, String> {
        // Determine the template directory path within the repo
        let template_dir = subpath.unwrap_or("templates");
        let base_url = format!(
            "https://raw.githubusercontent.com/{}/{}/{}/{}",
            owner, repo, branch, template_dir
        );
        let manifest_url = format!("{}/manifest.toml", base_url);

        // Fetch manifest
        let manifest_content = fetch_url(&manifest_url)?;
        let manifest: TemplateManifest = toml::from_str(&manifest_content)
            .map_err(|e| format!("Failed to parse manifest.toml: {}", e))?;

        // Create local cache directory
        let cache_name = format!("gh_{}_{}", owner, repo);
        let cache_dir = self.cache_root.join(&cache_name);
        fs::create_dir_all(&cache_dir)
            .map_err(|e| format!("Failed to create cache dir: {}", e))?;

        // Save manifest
        fs::write(cache_dir.join("manifest.toml"), &manifest_content)
            .map_err(|e| format!("Failed to write manifest: {}", e))?;

        // Source label file for identification
        let label = format!("gh:{}/{}", owner, repo);
        fs::write(cache_dir.join(".source"), &label)
            .map_err(|e| format!("Failed to write source label: {}", e))?;

        // Fetch each file in the template
        for file_entry in &manifest.files {
            let file_url = format!("{}/{}", base_url, file_entry.path);
            match fetch_url(&file_url) {
                Ok(content) => {
                    let file_path = cache_dir.join(&file_entry.path);
                    if let Some(parent) = file_path.parent() {
                        fs::create_dir_all(parent).ok();
                    }
                    let _ = fs::write(&file_path, &content);
                }
                Err(e) => {
                    eprintln!("  Warning: failed to fetch {}: {}", file_entry.path, e);
                }
            }
        }

        Ok(TemplatePackage {
            manifest,
            source: TemplateSource::GitHub {
                owner: owner.to_string(),
                repo: repo.to_string(),
                subpath: subpath.map(|s| s.to_string()),
                branch: branch.to_string(),
            },
            root: cache_dir,
        })
    }

    /// Pull from a raw URL (manifest must be at the URL root).
    fn pull_url(&self, url: &str) -> Result<TemplatePackage, String> {
        let content = fetch_url(url)?;
        let manifest: TemplateManifest = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse manifest from URL: {}", e))?;

        let cache_name = sanitize_name(&manifest.template.name);
        let cache_dir = self.cache_root.join(&cache_name);
        fs::create_dir_all(&cache_dir)
            .map_err(|e| format!("Failed to create cache dir: {}", e))?;

        fs::write(cache_dir.join("manifest.toml"), &content)
            .map_err(|e| format!("Failed to write manifest: {}", e))?;

        fs::write(cache_dir.join(".source"), url)
            .map_err(|e| format!("Failed to write source label: {}", e))?;

        Ok(TemplatePackage {
            manifest,
            source: TemplateSource::Url(url.to_string()),
            root: cache_dir,
        })
    }

    /// Load a template from the local cache by name.
    pub fn load(&self, name: &str) -> Result<TemplatePackage, String> {
        // Try direct name
        let dir = self.cache_root.join(name);
        if dir.exists() {
            let manifest_path = dir.join("manifest.toml");
            if manifest_path.exists() {
                let manifest = self.load_manifest(&manifest_path)?;
                return Ok(TemplatePackage {
                    manifest,
                    source: TemplateSource::Url("local".to_string()),
                    root: dir,
                });
            }
        }

        // Search all cached packages
        for pkg in self.list_local()? {
            if pkg.manifest.template.name == name {
                return Ok(pkg);
            }
        }

        Err(format!("Template '{}' not found in local cache", name))
    }

    /// Resolve a template source string to a local package (pull if needed).
    pub fn resolve(&self, source_str: &str) -> Result<TemplatePackage, String> {
        // Check if it's a local name first (no : or /)
        if !source_str.contains(':') && !source_str.contains('/') {
            // Try built-in packages first (always available, no cache needed)
            if let Ok(pkg) = self.pull_builtin(source_str) {
                return Ok(pkg);
            }
            // Then try local cache
            if let Ok(pkg) = self.load(source_str) {
                return Ok(pkg);
            }
            return Err(format!("Template '{}' not found. Available built-in: {}",
                source_str,
                self.list_builtin().join(", "),
            ));
        }

        // It's a remote source — parse and pull
        let source = TemplateSource::parse(source_str)?;
        self.pull(&source)
    }

    /// List available built-in template packages.
    pub fn list_builtin(&self) -> Vec<String> {
        crate::templates::list_builtin_packages()
    }

    pub fn pull_builtin(&self, name: &str) -> Result<TemplatePackage, String> {
        // Check that the package exists
        let builtins = self.list_builtin();
        if !builtins.contains(&name.to_string()) {
            return Err(format!("Built-in template '{}' not found. Available: {}", name, builtins.join(", ")));
        }

        // Load manifest from built-in
        let manifest_content = crate::templates::get_builtin_package_file(name, "manifest.toml")
            .ok_or_else(|| format!("Missing manifest.toml in built-in package '{}'", name))?;
        let manifest: TemplateManifest = toml::from_str(&manifest_content)
            .map_err(|e| format!("Failed to parse manifest: {}", e))?;

        // Create cache directory
        let cache_dir = self.cache_root.join(name);
        fs::create_dir_all(&cache_dir)
            .map_err(|e| format!("Failed to create cache dir: {}", e))?;

        // Save manifest
        fs::write(cache_dir.join("manifest.toml"), &manifest_content)
            .map_err(|e| format!("Failed to write manifest: {}", e))?;

        // Copy files from built-in to cache
        for file_entry in &manifest.files {
            if let Some(content) = crate::templates::get_builtin_package_file(name, &file_entry.path) {
                let file_path = cache_dir.join(&file_entry.path);
                if let Some(parent) = file_path.parent() {
                    fs::create_dir_all(parent).ok();
                }
                fs::write(&file_path, &content).ok();
            }
        }

        Ok(TemplatePackage {
            manifest,
            source: TemplateSource::Url(format!("builtin:{}", name)),
            root: cache_dir,
        })
    }

    /// Validate a template directory by checking its manifest.
    pub fn validate(path: &Path) -> Result<TemplateManifest, Vec<String>> {
        let mut errors = Vec::new();

        let manifest_path = if path.is_dir() {
            path.join("manifest.toml")
        } else {
            path.to_path_buf()
        };

        if !manifest_path.exists() {
            errors.push(format!(
                "manifest.toml not found at {}",
                manifest_path.display()
            ));
            return Err(errors);
        }

        let content = match fs::read_to_string(&manifest_path) {
            Ok(c) => c,
            Err(e) => {
                errors.push(format!("Failed to read manifest: {}", e));
                return Err(errors);
            }
        };

        let manifest: TemplateManifest = match toml::from_str(&content) {
            Ok(m) => m,
            Err(e) => {
                errors.push(format!("Invalid manifest: {}", e));
                return Err(errors);
            }
        };

        // Validate that referenced files exist
        for file_entry in &manifest.files {
            if let Some(parent) = manifest_path.parent() {
                let file_path = parent.join(&file_entry.path);
                if !file_path.exists() {
                    errors.push(format!(
                        "Referenced file '{}' not found at {}",
                        file_entry.path,
                        file_path.display()
                    ));
                }
            }
        }

        if errors.is_empty() {
            Ok(manifest)
        } else {
            Err(errors)
        }
    }

    fn load_manifest(&self, path: &Path) -> Result<TemplateManifest, String> {
        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read manifest: {}", e))?;
        toml::from_str(&content).map_err(|e| format!("Invalid manifest: {}", e))
    }

    fn source_label_from_path(path: &Path) -> String {
        let label_path = path.join(".source");
        if label_path.exists() {
            fs::read_to_string(&label_path).unwrap_or_else(|_| "local".to_string())
        } else {
            "local".to_string()
        }
    }
}

/// Fetch content from a URL via HTTPS.
fn fetch_url(url: &str) -> Result<String, String> {
    let response = ureq::get(url)
        .set("User-Agent", "dijiang-template-registry/1.0")
        .call()
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    if response.status() != 200 {
        return Err(format!(
            "HTTP {} fetching {}",
            response.status(),
            url
        ));
    }

    response
        .into_string()
        .map_err(|e| format!("Failed to read response body: {}", e))
}

/// Sanitize a name for use as a directory name.
fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_parse_github_source() {
        let src = TemplateSource::parse("gh:tiezhu/dijiang-templates").unwrap();
        match src {
            TemplateSource::GitHub {
                owner,
                repo,
                subpath,
                branch,
            } => {
                assert_eq!(owner, "tiezhu");
                assert_eq!(repo, "dijiang-templates");
                assert!(subpath.is_none());
                assert_eq!(branch, "main");
            }
            _ => panic!("Expected GitHub variant"),
        }
    }

    #[test]
    fn test_parse_github_with_subpath() {
        let src =
            TemplateSource::parse("gh:tiezhu/dijiang-templates/rust-api").unwrap();
        match src {
            TemplateSource::GitHub {
                owner,
                repo,
                subpath,
                branch,
            } => {
                assert_eq!(owner, "tiezhu");
                assert_eq!(repo, "dijiang-templates");
                assert_eq!(subpath.unwrap(), "rust-api");
                assert_eq!(branch, "main");
            }
            _ => panic!("Expected GitHub variant"),
        }
    }

    #[test]
    fn test_parse_github_with_branch() {
        let src = TemplateSource::parse("gh:tiezhu/dijiang-templates@develop").unwrap();
        match src {
            TemplateSource::GitHub {
                owner,
                repo,
                subpath,
                branch,
            } => {
                assert_eq!(owner, "tiezhu");
                assert_eq!(repo, "dijiang-templates");
                assert!(subpath.is_none());
                assert_eq!(branch, "develop");
            }
            _ => panic!("Expected GitHub variant"),
        }
    }

    #[test]
    fn test_parse_url_source() {
        let src = TemplateSource::parse(
            "https://raw.githubusercontent.com/tiezhu/dijiang-templates/main/templates/manifest.toml",
        )
        .unwrap();
        match src {
            TemplateSource::Url(url) => assert!(url.starts_with("https://")),
            _ => panic!("Expected Url variant"),
        }
    }

    #[test]
    fn test_parse_invalid() {
        assert!(TemplateSource::parse("").is_err());
        assert!(TemplateSource::parse("invalid").is_err());
        assert!(TemplateSource::parse("gh:").is_err());
    }

    #[test]
    fn test_validate_manifest() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let manifest_path = dir.path().join("manifest.toml");

        let manifest_content = r#"
[template]
name = "test-template"
version = "1.0.0"
description = "Test template"

[[files]]
path = "README.md"
description = "Readme file"
template = true
"#;

        let mut file = fs::File::create(&manifest_path)?;
        file.write_all(manifest_content.as_bytes())?;

        // Create the referenced file
        fs::write(dir.path().join("README.md"), "# Test")?;

        let result = TemplateRegistry::validate(dir.path());
        assert!(result.is_ok());
        let manifest = result.unwrap();
        assert_eq!(manifest.template.name, "test-template");
        assert_eq!(manifest.files.len(), 1);

        Ok(())
    }

    #[test]
    fn test_validate_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        let manifest_path = dir.path().join("manifest.toml");

        let manifest_content = r#"
[template]
name = "test-template"
version = "1.0.0"
description = "Test template"

[[files]]
path = "missing.md"
description = "This file doesn't exist"
"#;

        fs::write(&manifest_path, manifest_content).unwrap();

        let result = TemplateRegistry::validate(dir.path());
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("missing.md")));
    }

    #[test]
    fn test_validate_no_manifest() {
        let dir = tempfile::tempdir().unwrap();
        let result = TemplateRegistry::validate(dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_sanitize_name() {
        assert_eq!(sanitize_name("hello-world"), "hello-world");
        assert_eq!(sanitize_name("rust api"), "rust_api");
        assert_eq!(sanitize_name("a/b/c"), "a_b_c");
        assert_eq!(sanitize_name(""), "");
    }
}
