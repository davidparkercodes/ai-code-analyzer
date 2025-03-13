use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// The main error type for the application
#[derive(Debug, Error)]
pub enum AppError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    
    #[error("Path error: {0}")]
    #[allow(dead_code)]
    Path(String),
    
    #[error("File system error: path {path} - {message}")]
    FileSystem { path: PathBuf, message: String },
    
    #[error("Cache error: {0}")]
    #[allow(dead_code)]
    Cache(String),
    
    #[error("Metrics error: {0}")]
    Metrics(String),
    
    #[error("Dependency error: {0}")]
    Dependency(String),
    
    #[error("Analysis error: {0}")]
    Analysis(String),
    
    #[error("Style analysis error: {0}")]
    StyleAnalysis(String),
    
    #[error("Description error: {0}")]
    Description(String),
    
    #[error("Output formatting error: {0}")]
    #[allow(dead_code)]
    Formatting(String),
    
    #[error("AI error: {0}")]
    Ai(#[from] crate::ai::AiError),
    
    #[error("Internal error: {0}")]
    #[allow(dead_code)]
    Internal(String),
}

/// Helper function to convert a String error to AppError
#[allow(dead_code)]
pub fn to_app_error<E: ToString>(error: E, error_type: AppErrorType) -> AppError {
    match error_type {
        AppErrorType::Path => AppError::Path(error.to_string()),
        AppErrorType::Cache => AppError::Cache(error.to_string()),
        AppErrorType::Metrics => AppError::Metrics(error.to_string()),
        AppErrorType::Dependency => AppError::Dependency(error.to_string()),
        AppErrorType::Analysis => AppError::Analysis(error.to_string()),
        AppErrorType::StyleAnalysis => AppError::StyleAnalysis(error.to_string()),
        AppErrorType::Description => AppError::Description(error.to_string()),
        AppErrorType::Formatting => AppError::Formatting(error.to_string()),
        AppErrorType::Internal => AppError::Internal(error.to_string()),
    }
}

/// Helper enum to specify error type when converting
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum AppErrorType {
    Path,
    Cache,
    Metrics,
    Dependency,
    Analysis,
    StyleAnalysis,
    Description,
    Formatting,
    Internal,
}

/// Result type alias for the application
pub type AppResult<T> = Result<T, AppError>;

/// Helper function for handling errors in command modules
pub fn handle_command_error(error: &AppError) -> i32 {
    // Import at use site to avoid circular dependencies
    use crate::output::style;
    
    style::print_error(&error.to_string());
    1 // Return error exit code
}