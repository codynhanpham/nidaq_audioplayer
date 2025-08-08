use log::{info, trace, warn};
use pyo3::ffi::c_str;
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use serde::{Deserialize, Serialize};

use super::super::utils;

#[derive(Debug, Serialize)]
pub struct SysInfo {
    pub user: Option<String>,
    pub version: Option<String>,
    pub src_dir: Option<String>,
    pub env: Option<String>,
}
impl Default for SysInfo {
    fn default() -> Self {
        SysInfo {
            user: None,
            version: None,
            src_dir: None,
            env: None,
        }
    }
}

#[tauri::command]
pub async fn get_pyenv_sysinfo(app: tauri::AppHandle) -> SysInfo {
    let mut sysinfo = SysInfo::default();

    // Resolve the src-python directory
    let resource_dir = utils::taurithing::resource_path(&app, "src-python");
    if let Ok(path) = resource_dir {
        sysinfo.src_dir = Some(path);
    }

    Python::with_gil(|py| {
        // Try to get Python version
        if let Ok(sys) = py.import("sys") {
            if let Ok(version) = sys.getattr("version") {
                if let Ok(version_str) = version.extract::<String>() {
                    sysinfo.version = Some(version_str);
                }
            }
        }

        // Try to get user and virtual environment info
        if let Ok(os) = py.import("os") {
            if let Ok(locals) = [("os", os)].into_py_dict(py) {
                // Try to get username
                let code = c_str!("os.getenv('USER') or os.getenv('USERNAME') or 'Unknown'");
                if let Ok(user_result) = py.eval(code, None, Some(&locals)) {
                    if let Ok(user) = user_result.extract::<String>() {
                        sysinfo.user = Some(user);
                    }
                }

                // Try to get virtual environment
                if let Ok(venv_result) = py.eval(
                    c_str!("os.environ.get('VIRTUAL_ENV') or os.environ['VIRTUAL_ENV']"),
                    None,
                    Some(&locals),
                ) {
                    if let Ok(venv) = venv_result.extract::<Option<String>>() {
                        sysinfo.env = venv;
                    }
                }
                if sysinfo.env.is_none() {
                    if let Ok(sys) = py.import("sys") {
                        if let Ok(prefix) = sys.getattr("prefix") {
                            if let Ok(prefix_str) = prefix.extract::<String>() {
                                sysinfo.env = Some(prefix_str);
                            }
                        }
                    }
                }
            }
        }
    });

    // info!("Python environment sysinfo:\n{:?}", sysinfo);

    sysinfo
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DAQmxDevice {
    name: String,
    product_category: String,
    product_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DAQmxInfo {
    pub driver: Option<String>,
    pub devices: Vec<DAQmxDevice>,
}
impl Default for DAQmxInfo {
    fn default() -> Self {
        DAQmxInfo {
            driver: None,
            devices: Vec::new(),
        }
    }
}

#[tauri::command]
pub async fn get_nidaq_sysinfo(app: tauri::AppHandle) -> DAQmxInfo {
    let mut nidaq_info = DAQmxInfo::default();

    let resource_dir = utils::taurithing::resource_path(&app, "src-python");

    Python::with_gil(|py| -> PyResult<()> {
        let sys = py.import("sys")?;
        let path = sys.getattr("path")?;
        if let Ok(dir) = resource_dir {
            path.call_method1("append", (&dir,))?;
            // also append dir/installation/env and dir/installation/env/lib/site-packages
            let env_path = format!("{}/installation/env", &dir);
            path.call_method1("append", (&env_path,))?;
            let site_packages_path = format!("{}/lib/site-packages", &env_path);
            path.call_method1("append", (&site_packages_path,))?;
        }

        let code = c_str!(include_str!(
            "../../../src-python/src/sysinfo/nidaq_sysinfo.py"
        ));

        // Execute the Python code
        let nidaqmx_sysinfo = PyModule::from_code(
            py,
            code,
            c_str!("../../../src-python/src/sysinfo/nidaq_sysinfo.py"),
            c_str!("nidaq_sysinfo"),
        )?;
        let get_nidaq_info = nidaqmx_sysinfo.getattr("get_nidaq_sysinfo")?;
        let result: PyObject = get_nidaq_info.call0()?.into();

        // Extract and deserialize the JSON string
        let json_string: String = result.extract(py)?;

        // Deserialize the JSON string into the DAQmxInfo struct
        match serde_json::from_str::<DAQmxInfo>(&json_string) {
            Ok(info) => nidaq_info = info,
            Err(e) => eprintln!("Failed to deserialize DAQmxInfo: {}", e),
        }

        Ok(())
    })
    .unwrap_or_else(|e| {
        eprintln!("Python execution failed: {}", e);
    });

    // info!("NI-DAQmx sysinfo:\n{:?}", nidaq_info);

    nidaq_info
}
