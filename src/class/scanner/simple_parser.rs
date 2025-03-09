use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use log::{debug, warn};
use regex::Regex;
use lazy_static::lazy_static;

use crate::utils::file_utils;

/// A simplified class block representing a class found in a file
#[derive(Debug, Clone)]
pub struct ClassBlock {
    /// Name of the class
    pub name: String,
    
    /// Parent class name, if any
    pub parent: Option<String>,
    
    /// Path to the file where this class was found
    pub file_path: PathBuf,
}

/// A compatibility type to match cpp_parser::Block for easier migration
#[derive(Debug, Clone)]
pub struct Block {
    /// Name of the class
    pub name: Option<String>,
    
    /// Parent class name, if any
    pub parent: Option<String>,
    
    /// Content of the class block
    pub content: String,
    
    /// Nested blocks within this block
    pub children: Vec<Block>,
}

/// A simple parser that extracts class definitions from C++ files using regex
#[derive(Debug)]
pub struct SimpleParser {
    /// Whether to output verbose logs
    pub verbose: bool,
}

impl SimpleParser {
    /// Create a new simple parser
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }
    
    /// Parse a file and extract class definitions
    pub fn parse_file(&self, file_path: impl AsRef<Path>) -> Result<Vec<ClassBlock>> {
        let file_path = file_path.as_ref();
        debug!("Parsing file: {}", file_path.display());
        
        // Read the file content using file_utils
        let content = file_utils::read_file_to_string(file_path)?;
            
        self.parse_content(content, file_path)
    }
    
    /// Parse content and extract class definitions
    pub fn parse_content(&self, content: String, file_path: &Path) -> Result<Vec<ClassBlock>> {
        lazy_static! {
            // Match class definitions with optional inheritance
            // Handles both "class Name;" and "class Name: Parent {"
            static ref CLASS_RE: Regex = Regex::new(
                r"class\s+([A-Za-z0-9_]+)(?:\s*:\s*([A-Za-z0-9_]+))?[\s{;]"
            ).unwrap();
        }
        
        let mut classes = Vec::new();
        
        for cap in CLASS_RE.captures_iter(&content) {
            let class_name = cap[1].to_string();
            let parent_name = cap.get(2).map(|m| m.as_str().to_string());
            
            if self.verbose {
                debug!("Found class: {} with parent: {:?} in {}", 
                    class_name, parent_name, file_path.display());
            }
            
            classes.push(ClassBlock {
                name: class_name,
                parent: parent_name,
                file_path: file_path.to_path_buf(),
            });
        }
        
        debug!("Found {} classes in {}", classes.len(), file_path.display());
        Ok(classes)
    }
    
    /// Convert our ClassBlock to the compatibility Block type
    pub fn to_blocks(&self, class_blocks: Vec<ClassBlock>) -> Vec<Block> {
        class_blocks.into_iter()
            .map(|cb| Block {
                name: Some(cb.name),
                parent: cb.parent,
                content: String::new(), // We're not tracking content in the simplified version
                children: Vec::new(),   // We're not tracking nested classes in the simplified version
            })
            .collect()
    }
} 