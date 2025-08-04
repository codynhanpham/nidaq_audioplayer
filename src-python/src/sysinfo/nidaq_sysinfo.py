import json
import nidaqmx

def get_nidaq_sysinfo() -> str:
    """
    Retrieves information about the local NI-DAQmx system and its devices.
    Returns a dictionary with driver version and device details.
    """
    local_system = nidaqmx.system.System.local()
    driver_version = local_system.driver_version

    info = {
        "driver": f"DAQmx {driver_version.major_version}.{driver_version.minor_version}.{driver_version.update_version}",
    }
    # append device information
    info["devices"] = [
        {
            "name": str(device.name),
            "product_category": str(device.product_category.name),
            "product_type": str(device.product_type)
        }
        for device in local_system.devices
    ]

    # Convert the info dictionary to a JSON string
    return json.dumps(info, indent=0)