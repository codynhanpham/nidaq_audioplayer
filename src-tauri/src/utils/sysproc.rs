use sysinfo::{Pid, System};

pub fn kill_pid(pid: u32) -> Result<(), String> {
    let system = System::new_all();

    if let Some(process) = system.process(Pid::from_u32(pid as u32)) {
        if process.kill() {
            Ok(())
        } else {
            Err(format!("Failed to kill process with PID {}", pid))
        }
    } else {
        Err(format!("No process found with PID {}", pid))
    }
}
