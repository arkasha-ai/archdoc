//! Configuration management for ArchDoc
//!
//! This module handles loading and validating the archdoc.toml configuration file.

use serde::{Deserialize, Serialize};
use std::path::Path;
use crate::errors::ArchDocError;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct Config {
    #[serde(default)]
    pub project: ProjectConfig,
    #[serde(default)]
    pub scan: ScanConfig,
    #[serde(default)]
    pub python: PythonConfig,
    #[serde(default)]
    pub analysis: AnalysisConfig,
    #[serde(default)]
    pub output: OutputConfig,
    #[serde(default)]
    pub diff: DiffConfig,
    #[serde(default)]
    pub thresholds: ThresholdsConfig,
    #[serde(default)]
    pub rendering: RenderingConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub caching: CachingConfig,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    #[serde(default = "default_root")]
    pub root: String,
    #[serde(default = "default_out_dir")]
    pub out_dir: String,
    #[serde(default = "default_entry_file")]
    pub entry_file: String,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default)]
    pub name: String,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            root: default_root(),
            out_dir: default_out_dir(),
            entry_file: default_entry_file(),
            language: default_language(),
            name: String::new(),
        }
    }
}

fn default_root() -> String {
    ".".to_string()
}

fn default_out_dir() -> String {
    "docs/architecture".to_string()
}

fn default_entry_file() -> String {
    "ARCHITECTURE.md".to_string()
}

fn default_language() -> String {
    "python".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    #[serde(default = "default_include")]
    pub include: Vec<String>,
    #[serde(default = "default_exclude")]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub follow_symlinks: bool,
    #[serde(default = "default_max_file_size")]
    pub max_file_size: String,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            include: default_include(),
            exclude: default_exclude(),
            follow_symlinks: false,
            max_file_size: default_max_file_size(),
        }
    }
}

fn default_include() -> Vec<String> {
    vec!["src".to_string(), "app".to_string(), "tests".to_string()]
}

fn default_exclude() -> Vec<String> {
    vec![
        ".venv".to_string(),
        "venv".to_string(),
        "__pycache__".to_string(),
        ".git".to_string(),
        "dist".to_string(),
        "build".to_string(),
        ".mypy_cache".to_string(),
        ".ruff_cache".to_string(),
        ".pytest_cache".to_string(),
        "*.egg-info".to_string(),
    ]
}

fn default_max_file_size() -> String {
    "10MB".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonConfig {
    #[serde(default = "default_src_roots")]
    pub src_roots: Vec<String>,
    #[serde(default = "default_include_tests")]
    pub include_tests: bool,
    #[serde(default = "default_parse_docstrings")]
    pub parse_docstrings: bool,
    #[serde(default = "default_max_parse_errors")]
    pub max_parse_errors: usize,
}

impl Default for PythonConfig {
    fn default() -> Self {
        Self {
            src_roots: default_src_roots(),
            include_tests: default_include_tests(),
            parse_docstrings: default_parse_docstrings(),
            max_parse_errors: default_max_parse_errors(),
        }
    }
}

fn default_src_roots() -> Vec<String> {
    vec!["src".to_string(), ".".to_string()]
}

fn default_include_tests() -> bool {
    true
}

fn default_parse_docstrings() -> bool {
    true
}

fn default_max_parse_errors() -> usize {
    10
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisConfig {
    #[serde(default = "default_resolve_calls")]
    pub resolve_calls: bool,
    #[serde(default)]
    pub resolve_inheritance: bool,
    #[serde(default = "default_detect_integrations")]
    pub detect_integrations: bool,
    #[serde(default = "default_integration_patterns")]
    pub integration_patterns: Vec<IntegrationPattern>,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            resolve_calls: default_resolve_calls(),
            resolve_inheritance: false,
            detect_integrations: default_detect_integrations(),
            integration_patterns: default_integration_patterns(),
        }
    }
}

fn default_resolve_calls() -> bool {
    true
}

fn default_detect_integrations() -> bool {
    true
}

fn default_integration_patterns() -> Vec<IntegrationPattern> {
    vec![
        IntegrationPattern {
            type_: "http".to_string(),
            patterns: vec!["requests".to_string(), "httpx".to_string(), "aiohttp".to_string()],
        },
        IntegrationPattern {
            type_: "db".to_string(),
            patterns: vec![
                "sqlalchemy".to_string(),
                "psycopg".to_string(),
                "mysql".to_string(),
                "sqlite3".to_string(),
            ],
        },
        IntegrationPattern {
            type_: "queue".to_string(),
            patterns: vec![
                "celery".to_string(),
                "kafka".to_string(),
                "pika".to_string(),
                "redis".to_string(),
            ],
        },
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationPattern {
    #[serde(rename = "type")]
    pub type_: String,
    pub patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    #[serde(default)]
    pub single_file: bool,
    #[serde(default = "default_per_file_docs")]
    pub per_file_docs: bool,
    #[serde(default = "default_create_directories")]
    pub create_directories: bool,
    #[serde(default)]
    pub overwrite_manual_sections: bool,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            single_file: false,
            per_file_docs: default_per_file_docs(),
            create_directories: default_create_directories(),
            overwrite_manual_sections: false,
        }
    }
}

fn default_per_file_docs() -> bool {
    true
}

fn default_create_directories() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffConfig {
    #[serde(default = "default_update_timestamp_on_change_only")]
    pub update_timestamp_on_change_only: bool,
    #[serde(default = "default_hash_algorithm")]
    pub hash_algorithm: String,
    #[serde(default = "default_preserve_manual_content")]
    pub preserve_manual_content: bool,
}

impl Default for DiffConfig {
    fn default() -> Self {
        Self {
            update_timestamp_on_change_only: default_update_timestamp_on_change_only(),
            hash_algorithm: default_hash_algorithm(),
            preserve_manual_content: default_preserve_manual_content(),
        }
    }
}

fn default_update_timestamp_on_change_only() -> bool {
    true
}

fn default_hash_algorithm() -> String {
    "sha256".to_string()
}

fn default_preserve_manual_content() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdsConfig {
    #[serde(default = "default_critical_fan_in")]
    pub critical_fan_in: usize,
    #[serde(default = "default_critical_fan_out")]
    pub critical_fan_out: usize,
    #[serde(default = "default_high_complexity")]
    pub high_complexity: usize,
}

impl Default for ThresholdsConfig {
    fn default() -> Self {
        Self {
            critical_fan_in: default_critical_fan_in(),
            critical_fan_out: default_critical_fan_out(),
            high_complexity: default_high_complexity(),
        }
    }
}

fn default_critical_fan_in() -> usize {
    20
}

fn default_critical_fan_out() -> usize {
    20
}

fn default_high_complexity() -> usize {
    50
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderingConfig {
    #[serde(default = "default_template_engine")]
    pub template_engine: String,
    #[serde(default = "default_max_table_rows")]
    pub max_table_rows: usize,
    #[serde(default = "default_truncate_long_descriptions")]
    pub truncate_long_descriptions: bool,
    #[serde(default = "default_description_max_length")]
    pub description_max_length: usize,
}

impl Default for RenderingConfig {
    fn default() -> Self {
        Self {
            template_engine: default_template_engine(),
            max_table_rows: default_max_table_rows(),
            truncate_long_descriptions: default_truncate_long_descriptions(),
            description_max_length: default_description_max_length(),
        }
    }
}

fn default_template_engine() -> String {
    "handlebars".to_string()
}

fn default_max_table_rows() -> usize {
    100
}

fn default_truncate_long_descriptions() -> bool {
    true
}

fn default_description_max_length() -> usize {
    200
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default = "default_log_file")]
    pub file: String,
    #[serde(default = "default_log_format")]
    pub format: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            file: default_log_file(),
            format: default_log_format(),
        }
    }
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_file() -> String {
    "archdoc.log".to_string()
}

fn default_log_format() -> String {
    "compact".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachingConfig {
    #[serde(default = "default_caching_enabled")]
    pub enabled: bool,
    #[serde(default = "default_cache_dir")]
    pub cache_dir: String,
    #[serde(default = "default_max_cache_age")]
    pub max_cache_age: String,
}

impl Default for CachingConfig {
    fn default() -> Self {
        Self {
            enabled: default_caching_enabled(),
            cache_dir: default_cache_dir(),
            max_cache_age: default_max_cache_age(),
        }
    }
}

fn default_caching_enabled() -> bool {
    true
}

fn default_cache_dir() -> String {
    ".archdoc/cache".to_string()
}

fn default_max_cache_age() -> String {
    "24h".to_string()
}

impl Config {
    /// Validate the configuration for correctness.
    ///
    /// Checks that paths exist, values are parseable, and settings are sensible.
    pub fn validate(&self) -> Result<(), ArchDocError> {
        // Check project.root exists and is a directory
        let root = Path::new(&self.project.root);
        if !root.exists() {
            return Err(ArchDocError::ConfigError(format!(
                "project.root '{}' does not exist",
                self.project.root
            )));
        }
        if !root.is_dir() {
            return Err(ArchDocError::ConfigError(format!(
                "project.root '{}' is not a directory",
                self.project.root
            )));
        }

        // Check language is python
        if self.project.language != "python" {
            return Err(ArchDocError::ConfigError(format!(
                "project.language '{}' is not supported. Only 'python' is currently supported",
                self.project.language
            )));
        }

        // Check scan.include is not empty
        if self.scan.include.is_empty() {
            return Err(ArchDocError::ConfigError(
                "scan.include must not be empty — at least one directory must be specified".to_string(),
            ));
        }

        // Check python.src_roots exist relative to project.root
        for src_root in &self.python.src_roots {
            let path = root.join(src_root);
            if !path.exists() {
                return Err(ArchDocError::ConfigError(format!(
                    "python.src_roots entry '{}' does not exist (resolved to '{}')",
                    src_root,
                    path.display()
                )));
            }
        }

        // Parse max_cache_age
        parse_duration(&self.caching.max_cache_age).map_err(|e| {
            ArchDocError::ConfigError(format!(
                "caching.max_cache_age '{}' is not valid: {}. Use formats like '24h', '7d', '30m'",
                self.caching.max_cache_age, e
            ))
        })?;

        // Parse max_file_size
        parse_file_size(&self.scan.max_file_size).map_err(|e| {
            ArchDocError::ConfigError(format!(
                "scan.max_file_size '{}' is not valid: {}. Use formats like '10MB', '1GB', '500KB'",
                self.scan.max_file_size, e
            ))
        })?;

        Ok(())
    }

    /// Load configuration from a TOML file
    pub fn load_from_file(path: &Path) -> Result<Self, ArchDocError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ArchDocError::ConfigError(format!("Failed to read config file: {}", e)))?;
        
        toml::from_str(&content)
            .map_err(|e| ArchDocError::ConfigError(format!("Failed to parse config file: {}", e)))
    }
    
    /// Save configuration to a TOML file
    pub fn save_to_file(&self, path: &Path) -> Result<(), ArchDocError> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| ArchDocError::ConfigError(format!("Failed to serialize config: {}", e)))?;
        
        std::fs::write(path, content)
            .map_err(|e| ArchDocError::ConfigError(format!("Failed to write config file: {}", e)))
    }
}

/// Parse a duration string like "24h", "7d", "30m" into seconds.
pub fn parse_duration(s: &str) -> Result<u64, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("empty duration string".to_string());
    }

    let (num_str, suffix) = split_numeric_suffix(s)?;
    let value: u64 = num_str
        .parse()
        .map_err(|_| format!("'{}' is not a valid number", num_str))?;

    match suffix {
        "s" => Ok(value),
        "m" => Ok(value * 60),
        "h" => Ok(value * 3600),
        "d" => Ok(value * 86400),
        "w" => Ok(value * 604800),
        _ => Err(format!("unknown duration suffix '{}'. Use s, m, h, d, or w", suffix)),
    }
}

/// Parse a file size string like "10MB", "1GB", "500KB" into bytes.
pub fn parse_file_size(s: &str) -> Result<u64, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("empty file size string".to_string());
    }

    let (num_str, suffix) = split_numeric_suffix(s)?;
    let value: u64 = num_str
        .parse()
        .map_err(|_| format!("'{}' is not a valid number", num_str))?;

    let suffix_upper = suffix.to_uppercase();
    match suffix_upper.as_str() {
        "B" => Ok(value),
        "KB" | "K" => Ok(value * 1024),
        "MB" | "M" => Ok(value * 1024 * 1024),
        "GB" | "G" => Ok(value * 1024 * 1024 * 1024),
        _ => Err(format!("unknown size suffix '{}'. Use B, KB, MB, or GB", suffix)),
    }
}

fn split_numeric_suffix(s: &str) -> Result<(&str, &str), String> {
    let pos = s
        .find(|c: char| !c.is_ascii_digit())
        .ok_or_else(|| format!("no unit suffix found in '{}'", s))?;
    if pos == 0 {
        return Err(format!("no numeric value found in '{}'", s));
    }
    Ok((&s[..pos], &s[pos..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("24h").unwrap(), 86400);
        assert_eq!(parse_duration("7d").unwrap(), 604800);
        assert_eq!(parse_duration("30m").unwrap(), 1800);
        assert_eq!(parse_duration("60s").unwrap(), 60);
        assert!(parse_duration("abc").is_err());
        assert!(parse_duration("").is_err());
        assert!(parse_duration("10x").is_err());
    }

    #[test]
    fn test_parse_file_size() {
        assert_eq!(parse_file_size("10MB").unwrap(), 10 * 1024 * 1024);
        assert_eq!(parse_file_size("1GB").unwrap(), 1024 * 1024 * 1024);
        assert_eq!(parse_file_size("500KB").unwrap(), 500 * 1024);
        assert!(parse_file_size("abc").is_err());
        assert!(parse_file_size("").is_err());
    }

    #[test]
    fn test_validate_default_config() {
        // Default config with "." as root should validate if we're in a valid dir
        let config = Config::default();
        // This should work since "." exists and is a directory
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_bad_language() {
        let mut config = Config::default();
        config.project.language = "java".to_string();
        let err = config.validate().unwrap_err();
        assert!(err.to_string().contains("not supported"));
    }

    #[test]
    fn test_validate_empty_include() {
        let mut config = Config::default();
        config.scan.include = vec![];
        let err = config.validate().unwrap_err();
        assert!(err.to_string().contains("must not be empty"));
    }

    #[test]
    fn test_validate_bad_root() {
        let mut config = Config::default();
        config.project.root = "/nonexistent/path/xyz".to_string();
        let err = config.validate().unwrap_err();
        assert!(err.to_string().contains("does not exist"));
    }

    #[test]
    fn test_validate_bad_cache_age() {
        let mut config = Config::default();
        config.caching.max_cache_age = "invalid".to_string();
        let err = config.validate().unwrap_err();
        assert!(err.to_string().contains("not valid"));
    }

    #[test]
    fn test_validate_bad_file_size() {
        let mut config = Config::default();
        config.scan.max_file_size = "notasize".to_string();
        let err = config.validate().unwrap_err();
        assert!(err.to_string().contains("not valid"));
    }
}