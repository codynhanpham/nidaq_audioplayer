import type { DAQmxDevice, PySysInfo } from "$lib/applications/sysinfo";

export type StatusBarDataType = {
    // NI-DAQ stuff
    niDaqDriverVersion: string | null; // Version of the NI-DAQmx driver
    niDaqDevices: DAQmxDevice[] | null; // List of available NI-DAQmx devices
    niDaqSelectedDevice: DAQmxDevice | null; // Currently selected NI-DAQmx device

    // Python Websocket Server
    pywsPid: number | null; // Process ID of the Python Websocket Server
    pywsConnected: boolean | null; // Connection status of the Python Websocket Server
    pySysInfo: PySysInfo | null; // Information about the current Python environment
}

export const StatusBarData: StatusBarDataType = $state({
    // NI-DAQ stuff
    niDaqDriverVersion: null,
    niDaqDevices: null,
    niDaqSelectedDevice: null,

    // Python Websocket Server
    pywsPid: null,
    pywsConnected: null,
    pySysInfo: null
});