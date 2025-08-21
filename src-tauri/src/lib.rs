use std::{env, sync::Mutex};

use tauri::{Manager, RunEvent, WebviewWindow};
use tauri_plugin_window_state::{StateFlags, WindowExt};

pub mod appstate;
pub mod audio;
pub mod py_nidaqmx;
pub mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = app
                .get_webview_window("main")
                .expect("no main window")
                .set_focus();
        }))
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_websocket::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .level_for("symphonia_core::probe", log::LevelFilter::Off)
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::Webview,
                ))
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_handle = app.handle();

            let python_resource_dir = utils::taurithing::resource_path(&app_handle, "src-python");

            // Help the OS find the Python interpreter and packages, especially when user does not have Python installed globally
            // Well, and in some case, this also helps pyo3 to use the VENV created by src-python installer (though, not too consistent)
            if let Ok(dir) = python_resource_dir {
                app.manage(Mutex::new(appstate::AppData {
                    python_resource_dir: dir.clone(),
                    python_process: None,
                }));

                let env_path = format!("{}/installation/env", dir);
                let current_path = env::var_os("PATH").unwrap_or_default();
                let mut paths = env::split_paths(&current_path).collect::<Vec<_>>();
                if cfg!(target_os = "windows") {
                    paths.insert(0, format!("{}/installation/env/Scripts", &dir).into());
                    if std::path::Path::new(&format!("{}/installation/env", &dir)).exists() {
                        unsafe { env::set_var("PYTHONPATH", format!("{}/installation/env", &dir)) };
                        // unsafe { env::set_var("PYTHONHOME", format!("{}/installation/env", &dir)) }; // somehow, setting this makes pyo3 use correct VENV, but breaks all VENV packages
                    }
                } else {
                    paths.insert(0, format!("{}/installation/env/bin", &dir).into());
                    if std::path::Path::new(&format!("{}/installation/env/bin", &dir)).exists() {
                        unsafe {
                            env::set_var("PYTHONPATH", format!("{}/installation/env/bin", &dir))
                        };
                        // unsafe { env::set_var("PYTHONHOME", format!("{}/installation/env", &dir)) }; // same as above, setting this breaks all VENV packages
                    }
                }
                paths.insert(0, env_path.into());
                paths.insert(0, dir.into());
                unsafe {
                    env::set_var("PATH", env::join_paths(paths).unwrap());
                }

                // At the end of the setup, we can probably start the WebSocket server
                let process_handle = py_nidaqmx::pyws::start_ws(&app_handle);
                match process_handle {
                    Ok(_) => {
                        log::info!("Python WebSocket process spawned successfully");
                        // Store the process handle in the app state
                        let state = app.state::<Mutex<appstate::AppData>>();
                        state.lock().unwrap().python_process = Some(process_handle.unwrap());
                    }
                    Err(e) => log::error!("Failed to spawn Python WebSocket process: {}", e),
                }
            }

            // let window: WebviewWindow = app.get_webview_window("main").unwrap();
            // window.restore_state(StateFlags::all()).unwrap(); // restore state will be handled by the frontend

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            exit_app,
            py_nidaqmx::sysinfo::get_nidaq_sysinfo,
            py_nidaqmx::sysinfo::get_pyenv_sysinfo,
            py_nidaqmx::pyws::get_ws_pid,
            audio::glob_filter::filter_audio_files,
            audio::glob_filter::parse_dirs_from_paths,
            audio::glob_filter::flex_search_audio_files,
            audio::metadata::get_media_metadata,
            audio::multitrack_gen::audio_from_playlist,
            utils::datastore::save_audio_metadata,
            utils::datastore::load_audio_metadata,
            utils::datastore::calculate_audio_metadata_hash,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(move |_app_handle, _event| {
        match &_event {
            RunEvent::ExitRequested { api, code, .. } => {
                // Keep the event loop running even if all windows are closed
                // This allow us to catch tray icon events when there is no window
                // if we manually requested an exit (code is Some(_)) we will let it go through
                if code.is_none() {
                    api.prevent_exit();
                }
            }
            RunEvent::WindowEvent {
                event: tauri::WindowEvent::CloseRequested { api, .. },
                label,
                ..
            } => {
                println!("Closing window...");
                // run the window destroy manually just for fun :)
                // usually you'd show a dialog here to ask for confirmation or whatever
                api.prevent_close();

                // Check if python process is running and terminate it
                let state = _app_handle.state::<Mutex<appstate::AppData>>();
                if let Ok(mut app_data) = state.lock() {
                    if let Some(mut process) = app_data.python_process.take() {
                        if let Err(e) = process.kill() {
                            log::error!("Failed to terminate Python process: {}", e);
                        } else {
                            log::info!("Python process terminated successfully");
                        }
                    }
                }
                // Check if the python websocket server is running and close it
                let app_handle = _app_handle.clone();
                let window_label = label.clone();
                tauri::async_runtime::spawn(async move {
                    // Check if the python websocket server is running and close it
                    if let Ok(pid) = py_nidaqmx::pyws::get_ws_pid().await {
                        log::info!(
                            "WebSocket server (PID {}) still running, terminating...",
                            pid
                        );

                        match utils::sysproc::kill_pid(pid as u32) {
                            Ok(_) => log::info!("WebSocket server terminated successfully"),
                            Err(e) => log::error!("Failed to kill WebSocket server: {}", e),
                        }
                    }

                    // Close the window after cleanup
                    if let Some(window) = app_handle.get_webview_window(&window_label) {
                        let _ = window.destroy();
                    }

                    // End the app
                    app_handle.exit(0i32);
                });
            }
            _ => (),
        }
    })
}

#[tauri::command]
fn exit_app(app_handle: tauri::AppHandle) {
    // app_handle.exit(0i32); // This function kill the app process without triggering the cleanup logic!

    for (_, window) in app_handle.webview_windows() {
        let _ = window.close();
    }
}
