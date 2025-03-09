use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use log::{info, debug};

use crate::utils::file_utils;
use super::types::ClassDatabase;

/// Database storage operations
#[derive(Debug)]
pub struct DatabaseStorage {
    /// Path to the database file
    db_path: PathBuf,
}

impl DatabaseStorage {
    /// Create a new database storage with the given path
    pub fn new(db_path: impl AsRef<Path>) -> Self {
        Self {
            db_path: db_path.as_ref().to_path_buf(),
        }
    }
    
    /// Load the database from disk
    pub fn load(&self) -> Result<ClassDatabase> {
        let path = &self.db_path;
        debug!("Loading database from {}", path.display());
        
        if !path.exists() {
            info!("Database file does not exist, creating new database");
            return Ok(ClassDatabase::default());
        }
        
        let content = file_utils::read_file_to_string(path)?;
        
        let db: ClassDatabase = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse database file {}", path.display()))?;
        
        info!("Loaded database with {} classes", db.entries.len());
        Ok(db)
    }
    
    /// Save the database to disk
    pub fn save(&self, db: &ClassDatabase) -> Result<()> {
        let path = &self.db_path;
        debug!("Saving database to {}", path.display());
        
        // Create parent directory if needed using file_utils
        if let Some(parent) = path.parent() {
            file_utils::ensure_dir_exists(parent)?;
        }
        
        let content = serde_json::to_string_pretty(db)
            .context("Failed to serialize database")?;
        
        file_utils::write_string_to_file(path, &content)?;
        
        info!("Saved database with {} classes", db.entries.len());
        Ok(())
    }
    
    /// Check if the database file exists
    pub fn exists(&self) -> bool {
        self.db_path.exists()
    }
    
    /// Get the path to the database file
    pub fn path(&self) -> &Path {
        &self.db_path
    }
} 