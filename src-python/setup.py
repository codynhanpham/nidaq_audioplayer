# This script, along with the setup_*.bat / .sh were taken and modified based on ones from https://github.com/oobabooga/text-generation-webui

import argparse
import glob
import hashlib
import json
import os
import platform
import re
import signal
import site
import subprocess
import sys

# Define the required versions
PYTHON_VERSION = "3.12"
LIBSTDCXX_VERSION_LINUX = "12.1.0"

# Environment
script_dir = os.getcwd()
conda_env_path = os.path.join(script_dir, "installation", "env")

def signal_handler(sig, frame):
    sys.exit(0)


signal.signal(signal.SIGINT, signal_handler)


def is_linux():
    return sys.platform.startswith("linux")


def is_windows():
    return sys.platform.startswith("win")


def is_macos():
    return sys.platform.startswith("darwin")


def is_x86_64():
    return platform.machine() == "x86_64"


def is_installed():
    site_packages_path = None
    for sitedir in site.getsitepackages():
        if "site-packages" in sitedir and conda_env_path in sitedir:
            site_packages_path = sitedir
            break

    if site_packages_path:
        nidaqmxok = os.path.isfile(os.path.join(site_packages_path, 'nidaqmx', '__init__.py'))
        return nidaqmxok
    else:
        hascondaenv = os.path.isdir(conda_env_path)
        # also check for nidaqmx module
        nidaqmxok = os.path.isdir(os.path.join(conda_env_path, 'lib', f'python{PYTHON_VERSION}', 'site-packages', 'nidaqmx'))
        return hascondaenv and nidaqmxok


def check_env():
    # If we have access to conda, we are probably in an environment
    conda_exist = run_cmd("conda", environment=True, capture_output=True).returncode == 0
    if not conda_exist:
        print("Conda is not installed. Exiting...")
        sys.exit(1)

    # Ensure this is a new environment and not the base environment
    if os.environ.get("CONDA_DEFAULT_ENV", "") == "base":
        print("Create an environment for this project and activate it. Exiting...")
        sys.exit(1)


def check_nimex_driver():
    try:
        import nidaqmx

        local_system = nidaqmx.system.System.local()
        driver_version = local_system.driver_version

        print(
            "DAQmx {}.{}.{}".format(
                driver_version.major_version,
                driver_version.minor_version,
                driver_version.update_version,
            )
        )

        for device in local_system.devices:
            print(
                "Device Name: {}, Product Category: {}, Product Type: {}".format(
                    device.name, device.product_category.name, device.product_type
                )
            )
    except ImportError:
        print("nidaqmx-python module is not yet installed. Please re-run the setup script.")
        sys.exit(1)
    except nidaqmx.DaqError as e:
        print(f"DAQmx Error: {e}")
        sys.exit(1)
    except Exception as e:
        print(f"NI-DAQmx Installation Error: {e}\n\nPress Enter to try installing the NI-DAQmx driver, or Ctrl+C to exit.")
        input()
        run_cmd("python -m nidaqmx installdriver", assert_success=True, environment=True)
        check_nimex_driver()
        # sys.exit(1)

def clear_cache():
    run_cmd("conda clean -a -y", environment=True)
    run_cmd("python -m pip cache purge", environment=True)


def run_cmd(cmd, assert_success=False, environment=False, capture_output=False, env=None):
    # Use the conda environment
    if environment:
        if is_windows():
            conda_bat_path = os.path.join(script_dir, "installation", "conda", "condabin", "conda.bat")
            cmd = f'"{conda_bat_path}" activate "{conda_env_path}" >nul && {cmd}'
        else:
            conda_sh_path = os.path.join(script_dir, "installation", "conda", "etc", "profile.d", "conda.sh")
            cmd = f'. "{conda_sh_path}" && conda activate "{conda_env_path}" && {cmd}'

    # Set executable to None for Windows, bash for everything else
    executable = None if is_windows() else 'bash'

    # Run shell commands
    result = subprocess.run(cmd, shell=True, capture_output=capture_output, env=env, executable=executable)

    # Assert the command ran successfully
    if assert_success and result.returncode != 0:
        print(f"Command '{cmd}' failed with exit status code '{str(result.returncode)}'.\n\nExiting now.\nTry running the start/update script again.")
        sys.exit(1)

    return result


def print_big_message(message):
    message = message.strip()
    lines = message.split('\n')
    print("\n\n*******************************************************************")
    for line in lines:
        print("*", line)

    print("*******************************************************************\n\n")


def calculate_file_hash(file_path):
    p = os.path.join(script_dir, file_path)
    if os.path.isfile(p):
        with open(p, 'rb') as f:
            return hashlib.sha256(f.read()).hexdigest()
    else:
        return ''


def generate_alphabetic_sequence(index):
    result = ''
    while index >= 0:
        index, remainder = divmod(index, 26)
        result = chr(ord('A') + remainder) + result
        index -= 1

    return result


def get_user_choice(question, options_dict):
    print()
    print(question)
    print()

    for key, value in options_dict.items():
        print(f"{key}) {value}")

    print()

    choice = input("Input> ").upper()
    while choice not in options_dict.keys():
        print("Invalid choice. Please try again.")
        choice = input("Input> ").upper()

    return choice


def install_package():
    # Install Git and then some basic dependencies
    print_big_message("Installing some Python base dependencies.")
    run_cmd(f"conda install -y ninja git && python -m pip install -U pip", assert_success=True, environment=True)

    # Install the requirements
    update_requirements()


def update_requirements():
    requirements_base = os.path.join(".")
    # If multiple requirements files, determine the correct file here...
    requirement_file = "requirements.txt"
    requirements_file = os.path.join(requirements_base, requirement_file)

    print_big_message(f"Installing dependencies from file: {requirements_file}")

    # Install/update the project requirements
    run_cmd(f"python -m pip install -r {requirements_file} --upgrade", assert_success=True, environment=True)

    # Clean up
    clear_cache()


if __name__ == "__main__":
    # Verifies we are in a conda environment
    check_env()

    # parser = argparse.ArgumentParser(add_help=False)
    # parser.add_argument('--update-wizard', action='store_true', help='Launch a menu with update options.')
    # args, _ = parser.parse_known_args()

    if not is_installed():
        install_package()
        check_nimex_driver()
        print("Python packages are now installed and (should be) ready to go.")
        os.chdir(script_dir)
    else:
        # Check if the nidaqmx driver is installed and working
        check_nimex_driver()
        print("Python packages are already installed and (should be) ready to go.")
        os.chdir(script_dir)