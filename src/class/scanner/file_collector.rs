use std::path::{Path, PathBuf};
use anyhow::Result;
use log::{debug, trace};

use crate::utils::file_utils;

/// File collector for finding class files
#[derive(Debug, Default)]
pub struct FileCollector {
    /// Valid file extensions to collect
    valid_extensions: Vec<String>,
}

impl FileCollector {
    /// Create a new file collector with default settings
    pub fn new() -> Self {
        Self {
            valid_extensions: vec!["cpp".to_string(), "hpp".to_string()],
        }
    }
    
    /// Create a new file collector with custom file extensions
    pub fn with_extensions(extensions: Vec<String>) -> Self {
        Self {
            valid_extensions: extensions,
        }
    }
    
    /// Collect all files with valid extensions from the input directory
    pub fn collect_files(&self, input_dir: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
        let input_dir = input_dir.as_ref();
        debug!("Collecting files from directory: {}", input_dir.display());
        
        // Convert extensions to str slices for file_utils
        let extensions: Vec<&str> = self.valid_extensions.iter()
            .map(|s| s.as_str())
            .collect();
        
        // Use file_utils for consistent file collection
        let files = file_utils::get_files_with_extensions(input_dir, &extensions)?;
        
        debug!("Collected {} files for processing", files.len());
        Ok(files)
    }
    
    /// Add a valid file extension
    pub fn add_extension(&mut self, extension: &str) {
        if !self.valid_extensions.contains(&extension.to_string()) {
            self.valid_extensions.push(extension.to_string());
        }
    }
    
    /// Get the list of valid file extensions
    pub fn extensions(&self) -> &[String] {
        &self.valid_extensions
    }
} 