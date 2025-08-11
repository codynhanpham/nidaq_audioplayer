use super::super::utils;
use std::process::{Child, Command};
use std::os::windows::process::CommandExt;

use std::net::TcpListener;
use std::thread::spawn;
use serde::{Deserialize, Serialize};
use tungstenite::accept;


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
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        Command::new("cmd")
            .arg("/C")
            .arg(&entry_point)
            .creation_flags(CREATE_NO_WINDOW)
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
                "WS Server Entrypoint: {}",
                entry_point
            );
            Ok(child)
        }
        Err(e) => Err(format!("Failed to start WebSocket server: {}", e)),
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
struct PIDData {
    pid: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PIDWebSocketResponse {
    id: Option<String>,
    timestamp: Option<u64>,
    lastmsg: Option<String>,
    status: Option<String>,
    data: Option<PIDData>,
    completed: Option<bool>,
}

// response = {
//     "id": str(uuid.uuid4()),
//     "timestamp": int(time.time() * 1000),  # milliseconds
//     "lastmsg": self.last_message_id,
//     "status": status,
//     "data": data,
//     "completed": completed
// }
// pid_data = {
//     "pid": pid,
// }


/// Make a WS connection to the WebSocket server at `localhost:21749` and send #!pid message
/// Returns the PID of the WebSocket server process
#[tauri::command]
pub fn get_ws_pid() -> Result<u32, String> {
    let url = "ws://localhost:21749";
    let mut websocket = match tungstenite::connect(url) {
        Ok((ws, _)) => ws,
        Err(e) => return Err(format!("Failed to connect to WebSocket server: {}", e)),
    };

    let pid_message = r#"
    {
        "task": "pid"
    }
    "#;
    websocket.send(tungstenite::Message::Text(pid_message.to_string().into()))
        .map_err(|e| format!("Failed to send message to WebSocket server: {}", e))?;

    let response = match websocket.read() {
        Ok(msg) => msg,
        Err(e) => return Err(format!("Failed to read message from WebSocket server: {}", e)),
    };
    // let pid: u32 = response.to_string()
    //     .trim()
    //     .parse()
    //     .map_err(|e| format!("Failed to parse PID from WebSocket response: {}", e))?;
    // Parse response json
    let response: PIDWebSocketResponse = serde_json::from_str(&response.to_string())
        .map_err(|e| format!("Failed to parse PID from WebSocket response: {}", e))?;

    let pid = response.data.as_ref().map(|data| data.pid).unwrap_or(0);

    // close the WebSocket connection
    if let Err(e) = websocket.close(None) {
        log::warn!("Failed to close WebSocket connection: {}", e);
    }
    
    Ok(pid)
}