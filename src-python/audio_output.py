# filepath: audio_output.py
### AUDIO FILE OUTPUT EXAMPLE

import time
import numpy as np
import soundfile as sf

import nidaqmx as ni
from nidaqmx import stream_readers
from nidaqmx import stream_writers

from nidaqmx.constants import AcquisitionType, LineGrouping, ProductCategory

dev_name = 'Dev1'  # < remember to change to your device name, and channel input names below.
ao0 = '/ao0'
ao1 = '/ao1'
ai0 = '/ai0'
ai1 = '/ai1'

fs = 44100  # sample rate for input and output (will be overridden by audio file sample rate)
audio_file_path = "D:/JOBS/WashU_Neuroscience/Behavior/WU-SMAC/PlacePreference/Stimuli/playlists/calibration-sweep__linear_220.0Hz_140000.0Hz_10000ms_60.0dB_@300000Hz.wav"  # path to your audio file

NR_OF_CHANNELS = 2  # this script supports only 2 I/O channels
frames_per_buffer = 10  # nr of frames fitting into the buffer of each measurement channel
samples_per_frame = 8192

read_buffer = np.zeros((NR_OF_CHANNELS, samples_per_frame), dtype=np.float64)
voltage_scale = 0.1  # scale output voltage to max

# Add a flag to track audio completion
audio_completed = False


def get_terminal_name_with_dev_prefix(task: ni.Task, terminal_name: str) -> str:
    """Gets the terminal name with the device prefix."""
    for device in task.devices:
        if device.product_category not in [
            ProductCategory.C_SERIES_MODULE,
            ProductCategory.SCXI_MODULE,
        ]:
            return f"/{device.name}/{terminal_name}"
    raise RuntimeError("Suitable device not found in task.")


def audio_generator(file_path):
    """Generator that yields audio chunks from the input file."""
    try:
        data, sample_rate = sf.read(file_path, dtype='float64')
        global fs
        fs = int(sample_rate)
        
        # Ensure we have 2 channels
        if data.ndim == 1:
            # Mono - duplicate to stereo
            data = np.column_stack((data, data))
        elif data.shape[1] > 2:
            # More than 2 channels - take first 2
            data = data[:, :2]
        
        # Transpose to match expected format (channels, samples)
        data = data.T
        
        # Scale to voltage range
        data = data * voltage_scale
        
        # Yield chunks of the specified size
        total_samples = data.shape[1]
        current_pos = 0
        
        while current_pos < total_samples:
            end_pos = min(current_pos + samples_per_frame, total_samples)
            chunk_size = end_pos - current_pos
            
            data_frame = np.zeros((NR_OF_CHANNELS, samples_per_frame), dtype=np.float64)
            data_frame[:, :chunk_size] = data[:, current_pos:end_pos]
            
            print(f"{100 * current_pos / total_samples:.2f}% of audio file.")
            yield data_frame
            current_pos = end_pos
            # print current percentage of audio processed
        
        # Audio has ended - generator will stop here
        return
                
    except Exception as e:
        print(f"Error reading audio file: {e}")
        # Return empty generator on error
        return


def writing_task_callback(task_idx, event_type, num_samples, callback_data):
    """Callback for writing audio data to output buffer."""
    global audio_completed
    try:
        writer.write_many_sample(next(callback_data), timeout=10.0)
    except StopIteration:
        # Audio generator has finished
        audio_completed = True
        # Write silence for remaining buffer
        writer.write_many_sample(np.zeros((NR_OF_CHANNELS, samples_per_frame), dtype=np.float64), timeout=10.0)
    return 0


def reading_task_callback(task_idx, event_type, num_samples, callback_data=None):
    """Callback for reading input data."""
    reader.read_many_sample(read_buffer, num_samples, timeout=ni.constants.WAIT_INFINITELY)
    return 0


# Load audio file and get sample rate
try:
    info = sf.info(audio_file_path)
    fs = int(info.samplerate)
    print(f"Audio file: {audio_file_path}")
    print(f"Sample rate: {fs} Hz")
    print(f"Channels: {info.channels}")
    print(f"Duration: {info.duration:.2f} seconds")
except Exception as e:
    print(f"Error loading audio file info: {e}")
    print("Using default sample rate and silence")

# Update timebase with actual sample rate
timebase = np.arange(samples_per_frame) / fs

with ni.Task() as ao_task, ni.Task() as ai_task, ni.Task() as do_task:

    ai_args = {'min_val': -10,
               'max_val': 10,
               'terminal_config': ni.constants.TerminalConfiguration.RSE}

    ai_task.ai_channels.add_ai_voltage_chan(dev_name+ai0, **ai_args)
    ai_task.ai_channels.add_ai_voltage_chan(dev_name+ai1, **ai_args)
    ai_task.timing.cfg_samp_clk_timing(rate=fs, sample_mode=ni.constants.AcquisitionType.CONTINUOUS)
    ai_task.triggers.start_trigger.cfg_dig_edge_start_trig("ao/StartTrigger", trigger_edge=ni.constants.Edge.RISING)
    ai_task.in_stream.input_buf_size = samples_per_frame * frames_per_buffer * NR_OF_CHANNELS

    ao_args = {'min_val': -10,
               'max_val': 10}

    ao_task.ao_channels.add_ao_voltage_chan(dev_name+ao0, **ao_args)
    ao_task.ao_channels.add_ao_voltage_chan(dev_name+ao1, **ao_args)
    ao_task.timing.cfg_samp_clk_timing(rate=fs, sample_mode=ni.constants.AcquisitionType.CONTINUOUS)
    ao_task.out_stream.output_buf_size = samples_per_frame * frames_per_buffer * NR_OF_CHANNELS

    reader = stream_readers.AnalogMultiChannelReader(ai_task.in_stream)
    writer = stream_writers.AnalogMultiChannelWriter(ao_task.out_stream)

    terminal_name = get_terminal_name_with_dev_prefix(ao_task, "ao/SampleClock")

    # DIGITAL PULSE FOR EXTERNAL TRIGGERING
    do_task.do_channels.add_do_chan(dev_name + '/port0/line0',
                                    line_grouping=LineGrouping.CHAN_PER_LINE)
    do_task.do_channels.add_do_chan(dev_name + '/port0/line1',
                                    line_grouping=LineGrouping.CHAN_PER_LINE)
    do_task.write([False, False])

    # Create audio generator and fill output buffer
    output_frame_generator = audio_generator(audio_file_path)
    for _ in range(frames_per_buffer):
        writer.write_many_sample(next(output_frame_generator), timeout=1)

    ai_task.register_every_n_samples_acquired_into_buffer_event(samples_per_frame, reading_task_callback)
    ao_task.register_every_n_samples_transferred_from_buffer_event(
        samples_per_frame, lambda *args: writing_task_callback(*args[:-1], output_frame_generator))

    input("Press Enter to start audio playback...")

    ai_task.start()  # arms ai but does not trigger
    ao_task.start()  # triggers both ao and ai simultaneously
    do_task.write([True, True])

    time.sleep(0.01)      # 10ms pulse
    # do_task.write([False, False])  # Set LOW

    # Wait for audio to complete or user input
    print("Audio playing... Press CTRL+C to stop early or wait for audio to finish.")
    
    # Wait for either user input or audio completion
    try:
        while not audio_completed:
            time.sleep(0.05)
        ai_task.stop()
        ao_task.stop()
        do_task.write([False, False])
        do_task.stop()
        print("\nAudio playback completed!")
    except KeyboardInterrupt:
        ai_task.stop()
        ao_task.stop()
        do_task.write([False, False])
        do_task.stop()
        print("\nAudio playback stopped early.")
    except:
        pass