use std::sync::Mutex;
use tauri::{Builder, Manager};

#[derive(Default)]
pub struct AppData {
    pub python_resource_dir: String,
    pub python_process: Option<std::process::Child>,
}
