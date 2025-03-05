pub mod class;
pub mod database;
pub mod utils;

// Re-export main types and functions for easier access
pub use class::types::{ProcessedClass, ClassScanStats};
pub use class::scanner::ClassScanner;
pub use class::processor::ClassProcessor;
pub use class::types::ClassScanOptions;

pub use database::types::{ClassDatabase, ClassDatabaseStats};
pub use database::DatabaseOperations;
pub use database::QueryOptions;

// Re-export utility functions
pub use utils::file_utils;
