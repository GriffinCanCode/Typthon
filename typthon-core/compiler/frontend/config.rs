use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub check: CheckConfig,

    #[serde(default)]
    pub infer: InferConfig,

    #[serde(default)]
    pub errors: ErrorConfig,

    #[serde(default)]
    pub paths: PathsConfig,

    #[serde(default)]
    pub plugins: Vec<String>,

    #[serde(default)]
    pub overrides: HashMap<String, OverrideConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default = "default_false")]
    pub strict: bool,

    #[serde(default = "default_false")]
    pub allow_untyped_defs: bool,

    #[serde(default = "default_false")]
    pub allow_any: bool,

    #[serde(default = "default_true")]
    pub check_variance: bool,

    #[serde(default = "default_true")]
    pub check_generics: bool,

    #[serde(default = "default_false")]
    pub warn_redundant_casts: bool,

    #[serde(default = "default_false")]
    pub warn_unused_ignores: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default = "default_false")]
    pub aggressive: bool,

    #[serde(default = "default_true")]
    pub use_annotations: bool,

    #[serde(default)]
    pub max_iterations: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorConfig {
    #[serde(default = "default_100")]
    pub max_errors: usize,

    #[serde(default = "default_true")]
    pub show_suggestions: bool,

    #[serde(default = "default_true")]
    pub show_error_codes: bool,

    #[serde(default = "default_true")]
    pub color: bool,

    #[serde(default)]
    pub format: ErrorFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ErrorFormat {
    Default,
    Json,
    Compact,
    Verbose,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathsConfig {
    #[serde(default)]
    pub include: Vec<String>,

    #[serde(default)]
    pub exclude: Vec<String>,

    #[serde(default = "default_true")]
    pub follow_imports: bool,

    #[serde(default)]
    pub python_path: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverrideConfig {
    #[serde(flatten)]
    pub check: Option<CheckConfig>,

    #[serde(flatten)]
    pub infer: Option<InferConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            check: CheckConfig::default(),
            infer: InferConfig::default(),
            errors: ErrorConfig::default(),
            paths: PathsConfig::default(),
            plugins: Vec::new(),
            overrides: HashMap::new(),
        }
    }
}

impl Default for CheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            strict: false,
            allow_untyped_defs: false,
            allow_any: false,
            check_variance: true,
            check_generics: true,
            warn_redundant_casts: false,
            warn_unused_ignores: false,
        }
    }
}

impl Default for InferConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            aggressive: false,
            use_annotations: true,
            max_iterations: None,
        }
    }
}

impl Default for ErrorConfig {
    fn default() -> Self {
        Self {
            max_errors: 100,
            show_suggestions: true,
            show_error_codes: true,
            color: true,
            format: ErrorFormat::Default,
        }
    }
}

impl Default for ErrorFormat {
    fn default() -> Self {
        ErrorFormat::Default
    }
}

impl Default for PathsConfig {
    fn default() -> Self {
        Self {
            include: vec!["**/*.py".to_string()],
            exclude: vec![
                "**/node_modules/**".to_string(),
                "**/__pycache__/**".to_string(),
                "**/venv/**".to_string(),
                "**/.venv/**".to_string(),
            ],
            follow_imports: true,
            python_path: Vec::new(),
        }
    }
}

fn default_true() -> bool { true }
fn default_false() -> bool { false }
fn default_100() -> usize { 100 }

impl Config {
    /// Load configuration from .typyrc file
    pub fn load(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config: {}", e))?;

        Self::parse(&content)
    }

    /// Parse configuration from TOML string
    pub fn parse(content: &str) -> Result<Self, String> {
        toml::from_str(content)
            .map_err(|e| format!("Failed to parse config: {}", e))
    }

    /// Find and load configuration file from current directory or parents
    pub fn discover() -> Self {
        let mut current = std::env::current_dir().ok();

        while let Some(dir) = current {
            let config_path = dir.join(".typyrc");
            if config_path.exists() {
                if let Ok(config) = Self::load(&config_path) {
                    return config;
                }
            }

            // Try TOML extension
            let config_path = dir.join(".typyrc.toml");
            if config_path.exists() {
                if let Ok(config) = Self::load(&config_path) {
                    return config;
                }
            }

            current = dir.parent().map(|p| p.to_path_buf());
        }

        Self::default()
    }

    /// Get configuration for specific file (applying overrides)
    pub fn for_file(&self, path: &Path) -> Config {
        let mut config = self.clone();

        for (pattern, override_config) in &self.overrides {
            if Self::matches_glob(path, pattern) {
                if let Some(check) = &override_config.check {
                    config.check = check.clone();
                }
                if let Some(infer) = &override_config.infer {
                    config.infer = infer.clone();
                }
            }
        }

        config
    }

    fn matches_glob(path: &Path, pattern: &str) -> bool {
        // Simple but effective glob matching
        let path_str = path.to_string_lossy();
        let path_str = path_str.as_ref();

        if !pattern.contains('*') {
            return path_str.contains(pattern);
        }

        // Normalize path separators
        let path_normalized = path_str.replace('\\', "/");
        let pattern_normalized = pattern.replace('\\', "/");

        // Handle ** specially - it matches any number of path segments
        if pattern_normalized.contains("**") {
            let parts: Vec<&str> = pattern_normalized.split("**").collect();
            Self::match_double_star(&path_normalized, &parts)
        } else {
            // Simple * matching within a segment
            Self::match_single_star(&path_normalized, &pattern_normalized)
        }
    }

    fn match_single_star(text: &str, pattern: &str) -> bool {
        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.is_empty() {
            return true;
        }

        let mut pos = 0;
        for (i, &part) in parts.iter().enumerate() {
            if i == 0 {
                if !text[pos..].starts_with(part) {
                    return false;
                }
                pos += part.len();
            } else if i == parts.len() - 1 {
                return text[pos..].ends_with(part);
            } else if let Some(idx) = text[pos..].find(part) {
                pos += idx + part.len();
            } else {
                return false;
            }
        }
        true
    }

    fn match_double_star(text: &str, parts: &[&str]) -> bool {
        if parts.is_empty() {
            return true;
        }

        let first = parts[0];
        let last = parts[parts.len() - 1];

        // Check if first part matches start of path
        if !first.is_empty() {
            let first_trimmed = first.trim_end_matches('/');
            if !first_trimmed.is_empty() {
                if !text.starts_with(first_trimmed) {
                    return false;
                }
            }
        }

        // Check if last part matches end of path
        if !last.is_empty() {
            let last_trimmed = last.trim_start_matches('/');
            if !last_trimmed.is_empty() {
                if !Self::match_single_star(text, &format!("*{}", last_trimmed)) {
                    return false;
                }
            }
        }

        // Check middle parts exist somewhere in the path
        for &part in &parts[1..parts.len().saturating_sub(1)] {
            if !part.is_empty() {
                let part_trimmed = part.trim_matches('/');
                if !part_trimmed.is_empty() && !text.contains(part_trimmed) {
                    return false;
                }
            }
        }

        true
    }

    /// Check if a path should be checked based on include/exclude patterns
    pub fn should_check(&self, path: &Path) -> bool {
        // Check exclusions first
        for pattern in &self.paths.exclude {
            if Self::matches_glob(path, pattern) {
                return false;
            }
        }

        // Then check inclusions
        if self.paths.include.is_empty() {
            return true;
        }

        for pattern in &self.paths.include {
            if Self::matches_glob(path, pattern) {
                return true;
            }
        }

        false
    }

    /// Generate default configuration file content
    pub fn generate_default() -> String {
        toml::to_string_pretty(&Self::default())
            .unwrap_or_else(|_| String::from("# Failed to generate config"))
    }

    /// Save configuration to file
    pub fn save(&self, path: &Path) -> Result<(), String> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        fs::write(path, content)
            .map_err(|e| format!("Failed to write config: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.check.enabled);
        assert!(!config.check.strict);
    }

    #[test]
    fn test_parse_config() {
        let toml = r#"
[check]
enabled = true
strict = true

[infer]
enabled = true
aggressive = false

[errors]
max_errors = 50
"#;

        let config = Config::parse(toml).unwrap();
        assert!(config.check.enabled);
        assert!(config.check.strict);
        assert_eq!(config.errors.max_errors, 50);
    }

    #[test]
    fn test_should_check() {
        let config = Config::default();
        assert!(config.should_check(Path::new("src/main.py")));
        assert!(!config.should_check(Path::new("venv/lib/python3.11/site.py")));
    }
}

