//! File scanner for ArchDoc
//!
//! This module handles scanning the file system for Python files according to
//! the configuration settings.

use crate::config::Config;
use crate::errors::ArchDocError;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct FileScanner {
    config: Config,
}

impl FileScanner {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
    
    pub fn scan_python_files(&self, root: &Path) -> Result<Vec<PathBuf>, ArchDocError> {
        // Check if root directory exists
        if !root.exists() {
            return Err(ArchDocError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Root directory does not exist: {}", root.display())
            )));
        }
        
        if !root.is_dir() {
            return Err(ArchDocError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Root path is not a directory: {}", root.display())
            )));
        }
        
        let mut python_files = Vec::new();
        
        // Walk directory tree respecting include/exclude patterns
        for entry in WalkDir::new(root)
            .follow_links(self.config.scan.follow_symlinks)
            .into_iter() {
            
            let entry = entry.map_err(|e| {
                ArchDocError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to read directory entry: {}", e)
                ))
            })?;
            
            let path = entry.path();
            
            // Skip excluded paths
            if self.is_excluded(path) {
                if path.is_dir() {
                    continue;
                } else {
                    continue;
                }
            }
            
            // Include Python files
            if path.extension().and_then(|s| s.to_str()) == Some("py") {
                python_files.push(path.to_path_buf());
            }
        }
        
        Ok(python_files)
    }
    
    fn is_excluded(&self, path: &Path) -> bool {
        // Convert path to string for pattern matching
        let path_str = match path.to_str() {
            Some(s) => s,
            None => return false, // If we can't convert to string, don't exclude
        };
        
        // Check if path matches any exclude patterns
        for pattern in &self.config.scan.exclude {
            if path_str.contains(pattern) {
                return true;
            }
        }
        
        false
    }
}