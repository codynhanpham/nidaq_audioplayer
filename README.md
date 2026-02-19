# NI-DAQmx Media Player

A modern media player focusing on audio playback through unconventional backends

## Supported Devices
- [x] [National Instruments](https://www.ni.com/en/shop.html)' [NI-DAQmx](https://www.ni.com/en/support/downloads/drivers/download.ni-daq-mx.html) compatible devices
- [ ] Traditional audio interfaces (ASIO, WASAPI, etc.)
- [ ] COM devices (ESP32, microcontrollers, etc.)


## Features
> This is still technically a work in progress. The core features are ready as listed below, though some additional QOL improvements and extras are planned and in the works.
- [x] Library location customization + Import
- [x] Media playback
- [x] Playback control (play/pause/seek)
- [x] Volume control
- [x] Stereo flipping (swap Left/Right channels on stereo audio)
- [x] Embedded Chapter metadata parsing + Synching + Seeking (for FLAC and some WAV formats)
- [x] Multi-channel composite [track generator via YAML](./src-tauri/src/audio/multitrack_gen.template.yml) with sample-accurate chapter timestamps

In addition, the following options are set for more scientific uses:
- Synchronized 5V trigger on Digital Output (DO) `dio0` and `dio1`

## Extras
- `metadata_extractor` - A simple standalone CLI tool to extract metadata from audio files. Or simply use the subcommand `metadata` from the main `nidaq_audioplayer` executable. Either output to stdout (for piping) or save to a JSON file.


## Hardware setup
> This part focuses on setting up NI-DAQmx compatible devices as the audio output backend
### Audio
For any NI-DAQmx compatible device, audio outputs are sent via Analog Output Line (AO)

#### Mono Source
All AO lines from 0-3 play the same audio signal

#### Stereo Source
- Even lines (`/ao0`, `/ao2`, etc.) are assigned to audio channel 1 (LEFT)
- Odd lines (`/ao1`, `/ao3`, etc.) are assigned to audio channel 2 (RIGHT)

#### Multi-Channel Source
Each AO line plays its corresponding audio channel:
- `/ao0` -- audio Channel 1
- `/ao1` -- audio Channel 2
- `/ao2` -- audio Channel 3
- ... and so on

If there are more AO lines than audio channels, the audio assignment wraps around.

### TTL Sync
Digital Input/Output (DIO) ports `0` and `1` will be held HIGH (5V) while audio is playing and return to LOW (0V) when idle.

> ***Planned:*** I/O channels should be customizable in the future for both analog and digital ports.

## Installation
1. Run the [installation executable](https://github.com/codynhanpham/nidaq_audioplayer/releases) to install the main NI-DAQmx Audio Player program. **Keep note of your installation location.**
***The installation path, for now, must NOT contain spaces!!*** This is a limitation of miniconda.

2. Go to your installed path (by default, `%LOCALAPPDATA%/NI-DAQmxAudioPlayer`), navigate to `src-python`, and run the `setup_{platform}.bat|sh` file.
This will install the NI-DAQmx runtime backend, as well as prompting you to install the official NI-DAQmx driver if not already available.
The backend Python environment is unlikely to change and only needs to be installed once. Everything will be saved to `./installation/` and you can delete that folder for a full uninstallation when needed.

**Note:** Even though NI-DAQmx driver can be installed automatically as provided by the official Python `nidaqmx` package, it is still recommended to get the official [bundled installer from NI's website](https://www.ni.com/en/support/downloads/drivers/download.ni-daq-mx.html) to ensure you have the appropriate drivers for your specific hardware and enhanced compatibility with some Linux distributions.

### Updates
To update the application, simply download and run the latest installer from the [releases page](https://github.com/codynhanpham/nidaq_audioplayer/releases). The installer will automatically detect your existing installation, prompt to uninstall the old version, and install the new version in its place while keeping your existing library and settings intact.

You do **not** need to re-install the NI-DAQmx backend environment: new code files are patched automatically and the environment + dependencies can be reused without any issues.

### Uninstallation
To uninstall the application, simply run the uninstaller located in your installation directory (by default, `%LOCALAPPDATA%/NI-DAQmxAudioPlayer/uninstall.exe`) or use the "Add or Remove Programs" feature in Windows. To also remove your settings and library associations, check the `Delete the application data` option when prompted during uninstallation. Your source library files will not be deleted.

To fully remove the Python NI-DAQmx backend environment, manually delete the installation folder (by default, `%LOCALAPPDATA%/NI-DAQmxAudioPlayer/src-python/`), which contains all the files related to the Python environment and dependencies.


## Usage
After installation *(make sure to also install the NI-DAQmx backend as mentioned above!)*, simply run the `nidaq_audioplayer(.exe)` executable, or run the Desktop shortcut to launch.


## Why?

The project started due to the lack of cheap and/or versatile hardware support for audio playback at high sampling rates. The maximum frequency any audio system can produce is roughly half the minimum sampling rate across *all* links in the chain: the source audio file, the processing hardware, the digital-to-analog conversion, the preamplifier, the speakers, etc. From these, digital-to-analog conversion (DAC) is often the limiting factor. Most consumer audio interfaces are capped at 192,000 Hz sampling rate at 24 bit-per-sample: an extremely reasonable if not high quality for their target audience - human listeners whose hearing range is typically maxed out at around 20,000 Hz.

However, rodents, mice to be specific, have a much broader vocalization and hearing range with some recorded frequencies exceeding 100,000 Hz. Recording devices capable of capturing these frequencies exist (mostly in the bat research community) but are often extremely expensive, single-use (only recording/playback through 1 or 2 channels without additional I/O options), and locked into proprietary systems.

In our lab, when preparing for a project involving rodent vocalizations, we faced significant challenges in finding reasonably-priced suitable hardware for reliable high-frequency audio playback. At the same time, there was a National Instruments (NI) data acquisition (DAQ) box lying unused, which we thought could be used in place of a typical audio DAC: they have plenty of analog output channels that can easily do 250 kHz (with some up to 1,000 kHz) sampling rate and even more analog input channels and digital I/O options for triggering accessory devices (lights, camera, sensors, etc.).

And thus, this project was created. The goal is to create an interface that is compatible not just with NI-DAQmx devices, but also with other popular hardware platforms that have similar capabilities: if it has some analog output, it can play audio. It is still a bit of a challenge to find good and cheap hardware that can do high sampling rate digital-to-analog conversion, but second-hand NI hardware can often be found at reasonable prices and other microcontrollers are getting faster and more capable.


## Future Directions
- [ ] Migrate to [uv](https://docs.astral.sh/uv/reference/installer/) as the Python environment manager for the NI-DAQmx backend to simplify installation and management
- [ ] Complete the GUI with the remaining features (history management, playlist management, customizable I/O channels, etc.)
- [ ] Implement a Rust-based NI-DAQmx backend to eliminate the need for a separate Python environment


## Technical Notes
### NI-DAQmx Backend
At the moment, controlling NI-DAQmx compatible devices is done through the official Python `nidaqmx` package, which provides a high-level interface to interact with NI hardware. Communication between the GUI frontend and the Python NI-DAQmx backend is done through a local websocket connection where only short instructions and metadata are sent back and forth. Audio data streaming and processing is handled entirely by the Python websocket server, which also allows for more direct control over the audio streaming process and better performance compared to routing audio data through the frontend and or Tauri's Rust backend.

Short term goal is to streamline the installation process for the NI-DAQmx backend and make it as seamless as possible. Manually going to the `src-python` folder and running the setup script is a bit annoying, even if it's a one-time thing. Using miniconda as the virtual environment manager also makes it a bit clunky to auto start/stop the websocket server via Tauri on application launch/close. Migrating to [uv](https://docs.astral.sh/uv/reference/installer/) as the Python environment manager might be a good option to simplify this process and make it more robust across different platforms.

A longer term goal is to implement a Rust-based NI-DAQmx backend using some crates like [nicompiler_backend](https://docs.rs/nicompiler_backend/latest/nicompiler_backend/) or directly binding to the official NI-DAQmx C API. This would eliminate the need for a separate Python environment and websocket server, and allow for more direct integration with the Tauri backend. Will take a lot more time and effort to implement this, though...