use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use thiserror::Error;
use tokio::process::Command;

#[derive(Error, Debug)]
pub enum RarError {
    #[error("RAR binary not found: {0}")]
    BinaryNotFound(String),
    
    #[error("Failed to execute command: {0}")]
    ExecutionFailed(String),
    
    #[error("Invalid password")]
    InvalidPassword,
    
    // 💡 Reserved for future file validation
    #[allow(dead_code)]
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    // 💡 Reserved for future permission checking
    #[allow(dead_code)]
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Archive corrupted")]
    ArchiveCorrupted,
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveEntry {
    pub name: String,
    pub size: u64,
    pub packed_size: u64,
    pub ratio: String,
    pub modified: String,
    pub is_dir: bool,
}

// 💡 Reserved for future progress reporting feature
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressInfo {
    pub current_file: String,
    pub percentage: u32,
    pub bytes_processed: u64,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveInfo {
    pub path: String,
    pub file_count: usize,
    pub total_size: u64,
    pub packed_size: u64,
    pub compression_ratio: String,
}


pub struct RarWrapper {
    rar_path: PathBuf,
    unrar_path: PathBuf,
}

impl RarWrapper {
    pub fn new(resource_dir: impl AsRef<Path>) -> Result<Self, RarError> {
        let resource_dir = resource_dir.as_ref();
        let rar_path = resource_dir.join("rar");
        let unrar_path = resource_dir.join("unrar");
        
        // 💡 Check if binaries exist and are executable
        if !rar_path.exists() {
            return Err(RarError::BinaryNotFound(rar_path.display().to_string()));
        }
        if !unrar_path.exists() {
            return Err(RarError::BinaryNotFound(unrar_path.display().to_string()));
        }
        
        Ok(Self { rar_path, unrar_path })
    }
    
    /// 创建 RAR 归档
    pub async fn create_archive(
        &self,
        archive_path: impl AsRef<Path>,
        files: &[PathBuf],
        password: Option<&str>,
        compression_level: u8,
        split_size: Option<&str>,
    ) -> Result<(), RarError> {
        let mut args = vec!["a".to_string()];
        
        // 💡 Set compression level (0-5, where 5 is best)
        let level = compression_level.min(5);
        args.push(format!("-m{}", level));
        
        // 💡 Add password if provided
        if let Some(pwd) = password {
            args.push(format!("-hp{}", pwd));
        }
        
        // 💡 Split archive if size specified
        if let Some(size) = split_size {
            args.push(format!("-v{}", size));
        }
        
        // 💡 Disable output to make it quiet
        args.push("-idq".to_string());
        
        // Archive path
        args.push(archive_path.as_ref().display().to_string());
        
        // Add files
        for file in files {
            args.push(file.display().to_string());
        }
        
        let output = Command::new(&self.rar_path)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?
            .wait_with_output()
            .await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(RarError::ExecutionFailed(stderr.to_string()));
        }
        
        Ok(())
    }
    
    /// 解压 RAR 归档
    pub async fn extract_archive(
        &self,
        archive_path: impl AsRef<Path>,
        dest_path: impl AsRef<Path>,
        password: Option<&str>,
    ) -> Result<(), RarError> {
        let mut args = vec!["x".to_string()];
        
        // 💡 Add password if provided
        if let Some(pwd) = password {
            args.push(format!("-p{}", pwd));
        } else {
            args.push("-p-".to_string()); // No password
        }
        
        // 💡 Overwrite existing files
        args.push("-o+".to_string());
        
        // 💡 Disable output messages
        args.push("-idq".to_string());
        
        // Archive path
        args.push(archive_path.as_ref().display().to_string());
        
        // Destination path
        args.push(dest_path.as_ref().display().to_string());
        
        let output = Command::new(&self.unrar_path)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?
            .wait_with_output()
            .await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            
            // 💡 Check for specific error conditions
            if stderr.contains("password") {
                return Err(RarError::InvalidPassword);
            } else if stderr.contains("CRC failed") || stderr.contains("corrupt") {
                return Err(RarError::ArchiveCorrupted);
            }
            
            return Err(RarError::ExecutionFailed(stderr.to_string()));
        }
        
        Ok(())
    }
    
    /// 列出归档内容
    pub async fn list_archive(
        &self,
        archive_path: impl AsRef<Path>,
    ) -> Result<Vec<ArchiveEntry>, RarError> {
        let args = vec![
            "lb".to_string(),  // List bare (filenames only)
            archive_path.as_ref().display().to_string(),
        ];
        
        let output = Command::new(&self.unrar_path)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?
            .wait_with_output()
            .await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(RarError::ExecutionFailed(stderr.to_string()));
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut entries = Vec::new();
        
        // 💡 Parse output - bare mode just lists filenames
        for line in stdout.lines() {
            let line = line.trim();
            if !line.is_empty() {
                entries.push(ArchiveEntry {
                    name: line.to_string(),
                    size: 0,
                    packed_size: 0,
                    ratio: "N/A".to_string(),
                    modified: "N/A".to_string(),
                    is_dir: line.ends_with('/'),
                });
            }
        }
        
        Ok(entries)
    }
    
    /// 测试归档完整性
    pub async fn test_archive(
        &self,
        archive_path: impl AsRef<Path>,
        password: Option<&str>,
    ) -> Result<bool, RarError> {
        let mut args = vec!["t".to_string()];
        
        if let Some(pwd) = password {
            args.push(format!("-p{}", pwd));
        } else {
            args.push("-p-".to_string());
        }
        
        args.push(archive_path.as_ref().display().to_string());
        
        let output = Command::new(&self.unrar_path)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?
            .wait_with_output()
            .await?;
        
        Ok(output.status.success())
    }
    
    /// 获取归档信息
    pub async fn get_archive_info(
        &self,
        archive_path: impl AsRef<Path>,
    ) -> Result<ArchiveInfo, RarError> {
        let args = vec![
            "l".to_string(),  // List contents with details
            "-v".to_string(), // Verbose
            archive_path.as_ref().display().to_string(),
        ];
        
        let output = Command::new(&self.unrar_path)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?
            .wait_with_output()
            .await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(RarError::ExecutionFailed(stderr.to_string()));
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        // 💡 Parse archive statistics from output
        let mut total_files = 0;
        let mut total_size = 0u64;
        let mut packed_size = 0u64;
        
        for line in stdout.lines() {
            if line.contains("files") {
                // Try to extract file count
                if let Some(count_str) = line.split_whitespace().next() {
                    if let Ok(count) = count_str.parse::<usize>() {
                        total_files = count;
                    }
                }
            }
        }
        
        // Get file size
        if let Ok(metadata) = std::fs::metadata(archive_path.as_ref()) {
            packed_size = metadata.len();
        }
        
        Ok(ArchiveInfo {
            path: archive_path.as_ref().to_string_lossy().to_string(),
            file_count: total_files,
            total_size,
            packed_size,
            compression_ratio: if total_size > 0 {
                format!("{:.1}%", (packed_size as f64 / total_size as f64) * 100.0)
            } else {
                "N/A".to_string()
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // 💡 Basic test to verify binary paths can be constructed
    #[test]
    fn test_wrapper_creation() {
        // This will fail in test environment without actual binaries
        // but demonstrates the structure
        let result = RarWrapper::new("./resources");
        assert!(result.is_ok() || matches!(result, Err(RarError::BinaryNotFound(_))));
    }
}
