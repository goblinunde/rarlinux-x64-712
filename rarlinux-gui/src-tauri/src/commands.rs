use crate::rar_wrapper::{RarWrapper, ArchiveEntry};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

// 💡 Helper function to get RAR binaries path - works in both dev and packaged environments
fn get_rar_wrapper(app: &AppHandle) -> Result<RarWrapper, String> {
    // Try multiple paths in order of priority
    let mut possible_paths: Vec<PathBuf> = vec![
        PathBuf::from("/usr/lib/rarlinux-gui"),  // Installed location (RPM)
    ];
    
    // Add resource dir if available
    if let Ok(res_dir) = app.path().resource_dir() {
        possible_paths.push(res_dir);
    }
    
    for path in possible_paths {
        eprintln!("🔍 Trying RAR path: {}", path.display());
        if path.join("rar").exists() && path.join("unrar").exists() {
            eprintln!("✅ Found RAR binaries at: {}", path.display());
            return RarWrapper::new(path)
                .map_err(|e| format!("Failed to initialize RAR wrapper: {}", e));
        }
    }
    
    Err("RAR binaries not found in any expected location. Please reinstall the application.".to_string())
}

#[tauri::command]
pub async fn create_archive(
    app: AppHandle,
    archive_path: String,
    files: Vec<String>,
    password: Option<String>,
    compression_level: u8,
    split_size: Option<String>,
) -> Result<String, String> {
    let wrapper = get_rar_wrapper(&app)?;
    
    let file_paths: Vec<PathBuf> = files.into_iter().map(PathBuf::from).collect();
    
    wrapper
        .create_archive(
            &archive_path,
            &file_paths,
            password.as_deref(),
            compression_level,
            split_size.as_deref(),
        )
        .await
        .map_err(|e| format!("Failed to create archive: {}", e))?;
    
    Ok(format!("Archive created successfully: {}", archive_path))
}

#[tauri::command]
pub async fn extract_archive(
    app: AppHandle,
    archive_path: String,
    dest_path: String,
    password: Option<String>,
) -> Result<String, String> {
    let wrapper = get_rar_wrapper(&app)?;
    
    wrapper
        .extract_archive(&archive_path, &dest_path, password.as_deref())
        .await
        .map_err(|e| format!("Failed to extract archive: {}", e))?;
    
    Ok(format!("Archive extracted successfully to: {}", dest_path))
}

#[tauri::command]
pub async fn list_archive_contents(
    app: AppHandle,
    archive_path: String,
) -> Result<Vec<ArchiveEntry>, String> {
    let wrapper = get_rar_wrapper(&app)?;
    
    wrapper
        .list_archive(&archive_path)
        .await
        .map_err(|e| format!("Failed to list archive: {}", e))
}

#[tauri::command]
pub async fn test_archive(
    app: AppHandle,
    archive_path: String,
    password: Option<String>,
) -> Result<bool, String> {
    let wrapper = get_rar_wrapper(&app)?;
    
    wrapper
        .test_archive(&archive_path, password.as_deref())
        .await
        .map_err(|e| format!("Failed to test archive: {}", e))
}

#[tauri::command]
pub async fn get_archive_info(
    app: AppHandle,
    archive_path: String,
) -> Result<crate::rar_wrapper::ArchiveInfo, String> {
    let wrapper = get_rar_wrapper(&app)?;
    
    wrapper
        .get_archive_info(&archive_path)
        .await
        .map_err(|e| format!("Failed to get archive info: {}", e))
}
