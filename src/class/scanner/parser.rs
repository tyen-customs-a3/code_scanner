use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use std::thread;

use anyhow::{Result, Context, anyhow};
use log::{debug, warn, error, trace};

use crate::class::types::ClassScanOptions;
use crate::utils::file_utils;
use super::simple_parser::{SimpleParser, Block};

/// Class parser for parsing class files
#[derive(Debug)]
pub struct ClassParser {
    /// Configuration options for parsing
    options: ClassScanOptions,
    
    /// Output directory for logs and temporary files
    output_dir: PathBuf,
    
    /// Simple parser for actual parsing
    simple_parser: SimpleParser,
}

impl ClassParser {
    /// Create a new class parser with the given options
    pub fn new(options: ClassScanOptions, output_dir: impl AsRef<Path>) -> Self {
        Self {
            options: options.clone(),
            output_dir: output_dir.as_ref().to_path_buf(),
            simple_parser: SimpleParser::new(options.verbose_errors),
        }
    }
    
    /// Parse a file and return the blocks found in it
    pub fn parse_file(&self, file: impl AsRef<Path>) -> Result<Vec<Block>> {
        let file_path = file.as_ref();
        debug!("Parsing file: {}", file_path.display());
        
        // Read the file content using file_utils
        let content = file_utils::read_file_to_string(file_path)?;
        
        // Parse using the simple parser and convert to Block type
        let class_blocks = self.simple_parser.parse_content(content, file_path)?;
        let blocks = self.simple_parser.to_blocks(class_blocks);
        
        Ok(blocks)
    }
    
    /// Parse a file with a timeout and return the blocks found in it
    pub fn parse_file_with_timeout(&self, file: impl AsRef<Path>, timeout_seconds: u64) -> Result<(Vec<Block>, bool)> {
        let file_path = file.as_ref();
        let timeout = Duration::from_secs(timeout_seconds);
        
        debug!("Parsing file with timeout: {} ({} seconds)", file_path.display(), timeout_seconds);
        
        // Start timer
        let start_time = Instant::now();
        
        // Read file content using file_utils
        let content = match file_utils::read_file_to_string(file_path) {
            Ok(content) => content,
            Err(e) => {
                warn!("Failed to read file {}: {}", file_path.display(), e);
                return Err(anyhow!("Failed to read file: {}", e));
            }
        };
        
        // Parse content
        let parse_result = self.simple_parser.parse_content(content, file_path);
        
        // Check for timeout
        let elapsed = start_time.elapsed();
        let timed_out = elapsed > timeout;
        
        if timed_out {
            warn!("Parsing timed out for file: {} ({}s)", file_path.display(), elapsed.as_secs());
            return Ok((Vec::new(), true));
        }
        
        // Handle parse result
        match parse_result {
            Ok(class_blocks) => {
                let blocks = self.simple_parser.to_blocks(class_blocks);
                Ok((blocks, false))
            }
            Err(e) => {
                warn!("Failed to parse file {}: {}", file_path.display(), e);
                Err(anyhow!("Failed to parse file: {}", e))
            }
        }
    }
    
    /// Log error details for a file
    pub fn log_parse_error(&self, file: &Path, error: &impl std::fmt::Display, content: &str) {
        if !self.options.verbose_errors {
            return;
        }
        
        let file_name = file.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
            
        let error_dir = self.output_dir.join("error_logs");
        file_utils::ensure_dir_exists(&error_dir).unwrap_or_else(|_| {
            warn!("Failed to create error log directory: {}", error_dir.display());
        });
        
        let error_file = error_dir.join(format!("{}_error.log", file_name));
        let error_content = format!("Error parsing file: {}\n\nError: {}\n\nContent:\n{}", 
            file.display(), error, content);
            
        file_utils::write_string_to_file(&error_file, &error_content).unwrap_or_else(|_| {
            warn!("Failed to write error log to: {}", error_file.display());
        });
        
        debug!("Wrote error log to: {}", error_file.display());
    }
    
    /// Log timeout files
    pub fn log_timeout_files(&self, timeout_files: &[PathBuf], output_dir: &Path) {
        if timeout_files.is_empty() {
            return;
        }
        
        let timeout_dir = output_dir.join("timeout_logs");
        file_utils::ensure_dir_exists(&timeout_dir).unwrap_or_else(|_| {
            warn!("Failed to create timeout log directory: {}", timeout_dir.display());
        });
        
        let timeout_file = timeout_dir.join("timeout_files.log");
        let timeout_content = timeout_files.iter()
            .map(|f| f.display().to_string())
            .collect::<Vec<_>>()
            .join("\n");
            
        file_utils::write_string_to_file(&timeout_file, &timeout_content).unwrap_or_else(|_| {
            warn!("Failed to write timeout log to: {}", timeout_file.display());
        });
        
        debug!("Wrote {} timeout files to: {}", timeout_files.len(), timeout_file.display());
    }
} 