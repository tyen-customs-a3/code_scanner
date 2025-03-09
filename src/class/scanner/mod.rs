mod file_collector;
mod parser;
mod progress;
pub mod simple_parser;

use std::path::{Path, PathBuf};
use std::collections::HashSet;

use anyhow::{Result, Context};
use log::{debug, warn, info};
use rayon::prelude::*;

use crate::class::types::{ClassScanOptions, ScanErrors};

// Re-export from submodules
pub use file_collector::FileCollector;
pub use parser::ClassParser;
pub use progress::ProgressTracker;
pub use simple_parser::{SimpleParser, ClassBlock, Block};

/// Class scanner for finding and parsing class files
#[derive(Debug)]
pub struct ClassScanner {
    /// Configuration options for scanning
    options: ClassScanOptions,
    
    /// Output directory for logs and temporary files
    output_dir: PathBuf,
    
    /// File collector for finding class files
    file_collector: FileCollector,
    
    /// Class parser for parsing class files
    parser: ClassParser,
    
    /// Progress tracker for displaying progress
    progress_tracker: ProgressTracker,
    
    /// Tracks error files encountered during scanning
    error_files: HashSet<PathBuf>,
    
    /// Tracks timeout files encountered during scanning
    timeout_files: HashSet<PathBuf>,
}

impl ClassScanner {
    /// Create a new class scanner with the given options
    pub fn new(options: ClassScanOptions, output_dir: impl AsRef<Path>) -> Self {
        let output_path = output_dir.as_ref().to_path_buf();
        Self {
            options: options.clone(),
            output_dir: output_path.clone(),
            file_collector: FileCollector::new(),
            parser: ClassParser::new(options, output_path),
            progress_tracker: ProgressTracker::new(),
            error_files: HashSet::new(),
            timeout_files: HashSet::new(),
        }
    }
    
    /// Create a new class scanner with default options
    pub fn with_defaults(output_dir: impl AsRef<Path>) -> Self {
        Self::new(ClassScanOptions::default(), output_dir)
    }
    
    /// Collect files from a directory
    pub fn collect_files(&self, input_dir: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
        self.file_collector.collect_files(input_dir)
    }
    
    /// Parse a single file and return the blocks
    pub fn parse_file(&self, file: impl AsRef<Path>) -> Result<Vec<Block>> {
        self.parser.parse_file(file)
    }
    
    /// Parse a file with timeout
    pub fn parse_file_with_timeout(&self, file: impl AsRef<Path>) -> Result<(Vec<Block>, bool)> {
        self.parser.parse_file_with_timeout(file, self.options.parse_timeout_seconds)
    }
    
    /// Scan files in parallel and return the results
    pub fn scan_files_parallel(&mut self, files: &[PathBuf]) -> Result<Vec<(PathBuf, Vec<Block>)>> {
        // Create a thread pool for parallel processing
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(self.options.parallel_threads.unwrap_or_else(num_cpus::get))
            .build()?;
            
        // Thread-safe vector for collecting results
        let results = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let error_files = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        
        // Process files in parallel
        pool.install(|| {
            files.par_iter().for_each(|file_path| {
                match self.parser.parse_file(file_path) {
                    Ok(blocks) => {
                        results.lock().unwrap().push((file_path.clone(), blocks));
                    }
                    Err(err) => {
                        warn!("Failed to parse file {}: {}", file_path.display(), err);
                        error_files.lock().unwrap().push(file_path.clone());
                    }
                }
            });
        });
        
        // Update our error files
        for error_file in error_files.lock().unwrap().iter() {
            self.error_files.insert(error_file.clone());
        }
        
        // Extract results from the thread-safe container
        let scanned_files = results.lock().unwrap().clone();
        
        Ok(scanned_files)
    }
    
    /// Add a file to the error files list
    pub fn add_error_file(&mut self, file: impl AsRef<Path>) {
        let file_path = file.as_ref().to_path_buf();
        self.error_files.insert(file_path);
    }
    
    /// Add a file to the timeout files list
    pub fn add_timeout_file(&mut self, file: impl AsRef<Path>) {
        let file_path = file.as_ref().to_path_buf();
        self.timeout_files.insert(file_path);
    }
    
    /// Get the errors encountered during scanning
    pub fn get_scan_errors(&self) -> ScanErrors {
        ScanErrors {
            error_files: self.error_files.iter().cloned().collect(),
            timeout_files: self.timeout_files.iter().cloned().collect(),
        }
    }
} 