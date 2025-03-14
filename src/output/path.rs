use std::path::{Path, PathBuf};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::{Local, Datelike};
use crate::util::error::{AppError, AppResult};

pub fn create_output_path(command_name: &str, root_dir_name: &str, extension: &str) -> AppResult<PathBuf> {
    let base_dir = ensure_base_output_dir()?;
    let date_dir = ensure_date_subdirectory(&base_dir)?;
    let command_dir = ensure_command_subdirectory(&date_dir, command_name)?;
    let filename = generate_output_filename(root_dir_name, extension);
    
    Ok(command_dir.join(filename))
}

pub fn resolve_output_path(command_name: &str, path: &str, extension: &str) -> AppResult<PathBuf> {
    let dir_name = if path == "." {
        std::env::current_dir()
            .ok()
            .and_then(|path| path.file_name().and_then(|name| name.to_str().map(String::from)))
            .unwrap_or_else(|| "current_dir".to_string())
    } else {
        let path_buf = Path::new(path).to_path_buf();
        if path_buf.is_dir() {
            path_buf.file_name()
                .and_then(|name| name.to_str().map(String::from))
                .unwrap_or_else(|| "dir".to_string())
        } else if path.starts_with('/') {
            path_buf.file_name()
                .and_then(|name| name.to_str().map(String::from))
                .unwrap_or_else(|| "file".to_string())
        } else {
            path.to_string()
        }
    };
    
    create_output_path(command_name, &dir_name, extension)
}

pub fn ensure_base_output_dir() -> AppResult<PathBuf> {
    let output_dir = PathBuf::from("output");
    
    if !output_dir.exists() {
        fs::create_dir_all(&output_dir).map_err(|e| AppError::FileSystem {
            path: output_dir.clone(),
            message: format!("Failed to create output directory: {}", e)
        })?;
    }
    
    Ok(output_dir)
}

pub fn ensure_date_subdirectory(parent_dir: &Path) -> AppResult<PathBuf> {
    let now = Local::now();
    let date_dir_name = format!("{:04}-{:02}-{:02}", now.year(), now.month(), now.day());
    let date_dir = parent_dir.join(date_dir_name);
    
    if !date_dir.exists() {
        fs::create_dir_all(&date_dir).map_err(|e| AppError::FileSystem {
            path: date_dir.clone(),
            message: format!("Failed to create date directory: {}", e)
        })?;
    }
    
    Ok(date_dir)
}

pub fn ensure_command_subdirectory(parent_dir: &Path, command_name: &str) -> AppResult<PathBuf> {
    let command_dir = parent_dir.join(command_name);
    
    if !command_dir.exists() {
        fs::create_dir_all(&command_dir).map_err(|e| AppError::FileSystem {
            path: command_dir.clone(),
            message: format!("Failed to create command directory: {}", e)
        })?;
    }
    
    Ok(command_dir)
}

pub fn generate_output_filename(root_dir_name: &str, extension: &str) -> String {
    let dir_name = if root_dir_name == "." {
        std::env::current_dir()
            .ok()
            .and_then(|path| path.file_name().and_then(|name| name.to_str().map(String::from)))
            .unwrap_or_else(|| "current_dir".to_string())
    } else {
        Path::new(root_dir_name)
            .file_name()
            .and_then(|name| name.to_str().map(String::from))
            .unwrap_or_else(|| root_dir_name.to_string())
    };
    
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    format!("{}_{}.{}", dir_name, timestamp, extension)
}
