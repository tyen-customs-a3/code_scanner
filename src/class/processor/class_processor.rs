use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::{Result, Context};
use log::{warn, info, debug};
use rayon::prelude::*;

use crate::class::types::{ProcessedClass, ClassScanStats, ClassScanOptions, ClassScanResult, ScanErrors};
use crate::class::scanner::simple_parser::{SimpleParser, ClassBlock};
use crate::class::scanner::FileCollector;

/// Class processor responsible for collecting parsed classes
#[derive(Debug)]
pub struct ClassProcessor {
    /// Configuration options for processing
    options: ClassScanOptions,
    
    /// Output directory for logs and temporary files
    output_dir: PathBuf,
    
    /// Simple parser for extracting class definitions
    parser: SimpleParser,
    
    /// File collector for finding class files
    file_collector: FileCollector,
    
    /// Error tracking
    scan_errors: ScanErrors,
}

impl ClassProcessor {
    /// Create a new class processor with the given options
    pub fn new(options: ClassScanOptions, output_dir: impl AsRef<Path>) -> Self {
        Self {
            options: options.clone(),
            output_dir: output_dir.as_ref().to_path_buf(),
            parser: SimpleParser::new(options.verbose_errors),
            file_collector: FileCollector::new(),
            scan_errors: ScanErrors::default(),
        }
    }
    
    /// Create a new class processor with default options
    pub fn with_defaults(output_dir: impl AsRef<Path>) -> Self {
        Self::new(ClassScanOptions::default(), output_dir)
    }
    
    /// Process files and return the results
    pub fn process_files(&mut self, files: &[PathBuf]) -> Result<ClassScanResult> {
        info!("Processing {} files", files.len());
        
        // Limit the number of files if configured
        let files_to_process = if let Some(max_files) = self.options.max_files {
            if files.len() > max_files {
                warn!("Limiting to {} files out of {}", max_files, files.len());
                &files[0..max_files]
            } else {
                files
            }
        } else {
            files
        };
        
        // Configure parallel processing based on options
        let thread_count = self.options.parallel_threads.unwrap_or_else(|| {
            let available = num_cpus::get();
            let used = std::cmp::max(1, available.saturating_sub(1));
            debug!("Using {} threads for parallel processing (available: {})", used, available);
            used
        });
        
        rayon::ThreadPoolBuilder::new()
            .num_threads(thread_count)
            .build_global()
            .unwrap_or_else(|e| warn!("Failed to build thread pool: {}", e));
        
        // Thread-safe collection of error files
        let error_files = Arc::new(Mutex::new(Vec::new()));
        let timeout_files = Arc::new(Mutex::new(Vec::new()));
        
        // Process files in parallel
        let results: Vec<_> = files_to_process.par_iter()
            .map(|file| {
                match self.parser.parse_file(file) {
                    Ok(classes) => (file.clone(), classes, true, None),
                    Err(e) => {
                        warn!("Failed to parse file {}: {}", file.display(), e);
                        // Thread-safe update of error files
                        error_files.lock().unwrap().push(file.clone());
                        (file.clone(), Vec::new(), false, Some(e.to_string()))
                    }
                }
            })
            .collect();
        
        // Update the scan errors after parallel processing
        self.scan_errors.error_files = error_files.lock().unwrap().clone();
        self.scan_errors.timeout_files = timeout_files.lock().unwrap().clone();
        
        // Calculate statistics and convert to processed classes
        let mut stats = ClassScanStats::default();
        let mut all_classes = Vec::new();
        
        stats.total_files = results.len();
        stats.error_files = self.scan_errors.error_files.len();
        stats.error_file_paths = self.scan_errors.error_files.clone();
        stats.timeout_files = self.scan_errors.timeout_files.len();
        stats.timeout_file_paths = self.scan_errors.timeout_files.clone();
        
        for (file, classes, success, error) in results {
            if !success {
                continue;
            }
            
            if classes.is_empty() {
                stats.empty_files += 1;
                continue;
            }
            
            stats.files_with_classes += 1;
            stats.total_classes += classes.len();
            
            // Convert ClassBlock to ProcessedClass
            let processed_classes: Vec<ProcessedClass> = classes.into_iter()
                .map(|class| ProcessedClass {
                    name: class.name,
                    parent: class.parent,
                    properties: Vec::new(), // No properties in simplified version
                    file_path: Some(class.file_path),
                })
                .collect();
            
            all_classes.extend(processed_classes);
        }
        
        info!("Processed {} files, found {} classes", stats.total_files, stats.total_classes);
        
        Ok(ClassScanResult {
            classes: all_classes,
            stats,
        })
    }
    
    /// Scan a directory recursively for class files
    pub fn scan_directory(&mut self, input_dir: impl AsRef<Path>) -> Result<ClassScanResult> {
        let input_dir = input_dir.as_ref();
        info!("Scanning directory: {}", input_dir.display());
        
        // Use FileCollector to collect files instead of our own implementation
        let files = self.file_collector.collect_files(input_dir)?;
        info!("Found {} files to process", files.len());
        
        self.process_files(&files)
    }
    
    /// Scan specific files for classes
    pub fn scan_specific_files(&mut self, file_paths: &[PathBuf]) -> Result<ClassScanResult> {
        info!("Scanning {} specific files", file_paths.len());
        self.process_files(file_paths)
    }
    
    /// Get the scan errors
    pub fn get_scan_errors(&self) -> &ScanErrors {
        &self.scan_errors
    }
} 