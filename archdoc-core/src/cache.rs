//! Caching module for ArchDoc
//!
//! This module provides caching functionality to speed up repeated analysis
//! by storing parsed ASTs and analysis results.

use crate::config::Config;
use crate::errors::ArchDocError;
use crate::model::ParsedModule;
use std::path::Path;
use std::fs;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
struct CacheEntry {
    /// Timestamp when the cache entry was created
    created_at: DateTime<Utc>,
    /// Timestamp when the source file was last modified
    file_modified_at: DateTime<Utc>,
    /// The parsed module data
    parsed_module: ParsedModule,
}

pub struct CacheManager {
    config: Config,
    cache_dir: String,
}

impl CacheManager {
    pub fn new(config: Config) -> Self {
        let cache_dir = config.caching.cache_dir.clone();
        
        // Create cache directory if it doesn't exist
        if config.caching.enabled && !Path::new(&cache_dir).exists() {
            let _ = fs::create_dir_all(&cache_dir);
        }
        
        Self { config, cache_dir }
    }
    
    /// Get cached parsed module if available and not expired
    pub fn get_cached_module(&self, file_path: &Path) -> Result<Option<ParsedModule>, ArchDocError> {
        if !self.config.caching.enabled {
            return Ok(None);
        }
        
        let cache_key = self.get_cache_key(file_path);
        let cache_file = Path::new(&self.cache_dir).join(&cache_key);
        
        if !cache_file.exists() {
            return Ok(None);
        }
        
        // Read cache file
        let content = fs::read_to_string(&cache_file)
            .map_err(ArchDocError::Io)?;
        
        let cache_entry: CacheEntry = serde_json::from_str(&content)
            .map_err(|e| ArchDocError::AnalysisError(format!("Failed to deserialize cache entry: {}", e)))?;
        
        // Check if cache is expired
        let now = Utc::now();
        let cache_age = now.signed_duration_since(cache_entry.created_at);
        
        // Parse max_cache_age (simple format: "24h", "7d", etc.)
        let max_age_seconds = self.parse_duration(&self.config.caching.max_cache_age)?;
        
        if cache_age.num_seconds() > max_age_seconds as i64 {
            // Cache expired, remove it
            let _ = fs::remove_file(&cache_file);
            return Ok(None);
        }
        
        // Check if source file has been modified since caching
        let metadata = fs::metadata(file_path)
            .map_err(ArchDocError::Io)?;
        
        let modified_time = metadata.modified()
            .map_err(ArchDocError::Io)?;
        
        let modified_time: DateTime<Utc> = modified_time.into();
        
        if modified_time > cache_entry.file_modified_at {
            // Source file is newer than cache, invalidate cache
            let _ = fs::remove_file(&cache_file);
            return Ok(None);
        }
        
        Ok(Some(cache_entry.parsed_module))
    }
    
    /// Store parsed module in cache
    pub fn store_module(&self, file_path: &Path, parsed_module: ParsedModule) -> Result<(), ArchDocError> {
        if !self.config.caching.enabled {
            return Ok(());
        }
        
        let cache_key = self.get_cache_key(file_path);
        let cache_file = Path::new(&self.cache_dir).join(&cache_key);
        
        // Get file modification time
        let metadata = fs::metadata(file_path)
            .map_err(ArchDocError::Io)?;
        
        let modified_time = metadata.modified()
            .map_err(ArchDocError::Io)?;
        
        let modified_time: DateTime<Utc> = modified_time.into();
        
        let cache_entry = CacheEntry {
            created_at: Utc::now(),
            file_modified_at: modified_time,
            parsed_module,
        };
        
        let content = serde_json::to_string(&cache_entry)
            .map_err(|e| ArchDocError::AnalysisError(format!("Failed to serialize cache entry: {}", e)))?;
        
        fs::write(&cache_file, content)
            .map_err(ArchDocError::Io)
    }
    
    /// Generate cache key for a file path
    fn get_cache_key(&self, file_path: &Path) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        file_path.hash(&mut hasher);
        let hash = hasher.finish();
        
        format!("{:x}.json", hash)
    }
    
    /// Parse duration string like "24h" or "7d" into seconds
    fn parse_duration(&self, duration_str: &str) -> Result<u64, ArchDocError> {
        if duration_str.is_empty() {
            return Ok(0);
        }
        
        let chars: Vec<char> = duration_str.chars().collect();
        let (number_str, unit) = chars.split_at(chars.len() - 1);
        let number: u64 = number_str.iter().collect::<String>().parse()
            .map_err(|_| ArchDocError::AnalysisError(format!("Invalid duration format: {}", duration_str)))?;
        
        match unit[0] {
            's' => Ok(number), // seconds
            'm' => Ok(number * 60), // minutes
            'h' => Ok(number * 3600), // hours
            'd' => Ok(number * 86400), // days
            _ => Err(ArchDocError::AnalysisError(format!("Unknown duration unit: {}", unit[0]))),
        }
    }
    
    /// Clear all cache entries
    pub fn clear_cache(&self) -> Result<(), ArchDocError> {
        if Path::new(&self.cache_dir).exists() {
            fs::remove_dir_all(&self.cache_dir)
                .map_err(ArchDocError::Io)?;
            
            // Recreate cache directory
            fs::create_dir_all(&self.cache_dir)
                .map_err(ArchDocError::Io)?;
        }
        
        Ok(())
    }
}