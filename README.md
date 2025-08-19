# NI-DAQmx Media Player

A modern media player focusing on audio playback through unconventional backends

## Supported Devices
- [x] [National Instruments](https://www.ni.com/en/shop.html)' [NI-DAQmx](https://www.ni.com/en/support/downloads/drivers/download.ni-daq-mx.html) compatible devices
- [ ] Traditional audio interfaces (ASIO, WASAPI, etc.)
- [ ] COM devices (ESP32, microcontrollers, etc.)

## Why?

The project started due to the lack of cheap and/or versatile hardware support for audio playback at high sampling rates. The maximum frequency any audio system can produce is roughly half the minimum sampling rate across *all* links in the chain: the source audio file, the processing hardware, the digital-to-analog conversion, the preamplifier, the speakers, etc. From these, digital-to-analog conversion (DAC) is often the limiting factor. Most consumer audio interfaces are capped at 192,000 Hz sampling rate at 24 bit-per-sample: an extremely reasonable if not high quality for their target audience - human listeners whose hearing range is typically maxed out at around 20,000 Hz.

However, rodents, mice to be specific, have a much broader vocalization and hearing range with some recorded frequencies exceeding 100,000 Hz. Recording devices capable of capturing these frequencies exist (mostly in the bat research community) but are often extremely expensive, single-use (only recording/playback through 1 or 2 channels without additional I/O options), and locked into proprietary systems.

In our lab, when preparing for a project involving rodent vocalizations, we faced significant challenges in finding reasonably-priced suitable hardware for reliable high-frequency audio playback. At the same time, there was a National Instruments (NI) data acquisition (DAQ) box lying unused, which we thought could be used in place of a typical audio DAC: they have plenty of analog output channels that can easily do 250 kHz (with some up to 1,000 kHz) sampling rate and even more analog input channels and digital I/O options for triggering accessory devices (lights, camera, sensors, etc.).

And thus, this project was created. The goal is to create an interface that is compatible not just with NI-DAQmx devices, but also with other popular hardware platforms that have similar capabilities: if it has some analog output, it can play audio. It is still a bit of a challenge to find good and cheap hardware that can do high sampling rate digital-to-analog conversion, but second-hand NI hardware can often be found at reasonable prices and other microcontrollers are getting faster and more capable.