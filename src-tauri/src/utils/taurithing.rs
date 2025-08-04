use tauri::Manager;

#[tauri::command]
pub fn resource_path(app: &tauri::AppHandle, resource: &str) -> Result<String, String> {
    let src_dir = app
        .path()
        .resolve(resource, tauri::path::BaseDirectory::Resource);
    match src_dir {
        Ok(path) => {
            let canonical_path = std::fs::canonicalize(path).map_err(|e| e.to_string())?;
            let path_str = canonical_path.to_str().ok_or("Invalid UTF-8 sequence")?;
            // Strip Windows UNC prefix if present
            let clean_path = if path_str.starts_with("\\\\?\\") {
                &path_str[4..]
            } else {
                path_str
            };
            Ok(clean_path.to_string())
        }
        Err(e) => Err(e.to_string()),
    }
}
