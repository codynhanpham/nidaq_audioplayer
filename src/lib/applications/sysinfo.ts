import { invoke } from "@tauri-apps/api/core";

export type PySysInfo = {
    user: string | null,
    version: string | null,
    src_dir: string | null,
    env: string | null,
    pid: number | null
}

export type DAQmxDevice = {
    name: string,
    product_category: string,
    product_type: string,
}

export type DAQmxSysInfo = {
    driver: string | null, // Version of the NI-DAQmx driver
    devices: DAQmxDevice[] | null,
}

export async function pyenv_sysinfo(): Promise<PySysInfo> {
    // Call the Rust function to get Python environment information
    const result = await invoke("get_pyenv_sysinfo") as Object;
    const pid = await invoke("get_ws_pid") as number;
    return { ...result, pid } as PySysInfo;
}
export async function get_nidaq_sysinfo(): Promise<DAQmxSysInfo> {
    // Call the Rust function to get NI-DAQmx information
    const result = await invoke("get_nidaq_sysinfo") as DAQmxSysInfo;
    return result;
}