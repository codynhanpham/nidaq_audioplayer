use super::super::utils;
use std::process::{Child, Command};

/// Spawn child process triggering the start_ws_server.bat/sh script in src-python
#[tauri::command]
pub fn start_ws(app: &tauri::AppHandle) -> Result<Child, String> {
    let resource_dir = utils::taurithing::resource_path(app, "src-python")
        .map_err(|e| format!("Failed to resolve src-python directory: {}", e))?;

    let entry_point = if cfg!(target_os = "windows") {
        format!("{}/start_ws_server.bat", resource_dir)
    } else {
        format!("{}/start_ws_server.sh", resource_dir)
    };

    let process = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .arg("/C")
            .arg(&entry_point)
            .arg("&")
            .spawn()
    } else {
        Command::new("/bin/sh")
            .arg("-c")
            .arg(&entry_point)
            .arg("&")
            .spawn()
    };

    match process {
        Ok(child) => {
            log::info!(
                "Entrypoint: {}\nPython WebSocket process spawned with PID: {}",
                entry_point,
                child.id()
            );
            Ok(child)
        }
        Err(e) => Err(format!("Failed to start WebSocket server: {}", e)),
    }
}
