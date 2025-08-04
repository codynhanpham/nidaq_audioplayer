### WORKING EXAMPLE!!!


import time
import numpy as np

import nidaqmx as ni
from nidaqmx import stream_readers
from nidaqmx import stream_writers

from nidaqmx.constants import AcquisitionType, LineGrouping, ProductCategory

dev_name = 'Dev1'  # < remember to change to your device name, and channel input names below.
ao0 = '/ao0'
ao1 = '/ao1'
ai0 = '/ai0'
ai1 = '/ai1'

fs = 250000  # sample rate for input and output.
#NOTE: Depending on your hardware sample clock frequency and available dividers some sample rates may not be supported.
out_freq = 2000

NR_OF_CHANNELS = 2  # this scrip supports only 2 I/O channels (do not change)
frames_per_buffer = 10  # nr of frames fitting into the buffer of each measurement channel.
# NOTE  With my NI6211 it was necessary to override the default buffer size to prevent under/over run at high sample
# rates.

# read_buffer = np.zeros((NR_OF_CHANNELS, samples_per_frame), dtype=np.float64)
# timebase = np.arange(samples_per_frame) / fs

samples_per_frame = 8192
read_buffer = np.zeros((NR_OF_CHANNELS, samples_per_frame), dtype=np.float64)
timebase = np.arange(samples_per_frame) / fs 



def get_terminal_name_with_dev_prefix(task: ni.Task, terminal_name: str) -> str:
    """Gets the terminal name with the device prefix.

    Args:
        task: Specifies the task to get the device name from.
        terminal_name: Specifies the terminal name to get.

    Returns:
        Indicates the terminal name with the device prefix.
    """
    for device in task.devices:
        if device.product_category not in [
            ProductCategory.C_SERIES_MODULE,
            ProductCategory.SCXI_MODULE,
        ]:
            return f"/{device.name}/{terminal_name}"

    raise RuntimeError("Suitable device not found in task.")


def sine_generator():

    amplitudes = [1, 0.5]
    voltage_scale = 5 # scale output voltage to max +-5V

    frame_nr = 0
    data_frame = np.zeros((NR_OF_CHANNELS, samples_per_frame), dtype=np.float64)

    samples_per_period = fs / out_freq
    reminder_phase_rad = 2 * np.pi * (samples_per_frame % samples_per_period) / samples_per_period
    omega_t = 2 * np.pi * out_freq * timebase

    while True:
        phi = reminder_phase_rad * frame_nr
        for channel in range(NR_OF_CHANNELS):
            data_frame[channel] = amplitudes[channel]*np.sin(omega_t + phi) * voltage_scale
        yield data_frame
        frame_nr += 1


def writing_task_callback(task_idx, event_type, num_samples, callback_data):

    """This callback is called every time a defined amount of samples have been transferred from the device output
    buffer. This function is registered by register_every_n_samples_transferred_from_buffer_event and it must follow
    prototype defined in nidaqxm documentation.

    Args:
        task_idx (int): Task handle index value
        event_type (nidaqmx.constants.EveryNSamplesEventType): TRANSFERRED_FROM_BUFFER
        num_samples (int): Number of samples that was writen into the write buffer.
        callback_data (object): User data - I use this arg to pass signal generator object.
    """

    writer.write_many_sample(next(callback_data), timeout=10.0)

    # The callback function must return 0 to prevent raising TypeError exception.
    return 0


def reading_task_callback(task_idx, event_type, num_samples, callback_data=None):
    """This callback is called every time a defined amount of samples have been acquired into the input buffer. This
    function is registered by register_every_n_samples_acquired_into_buffer_event and must follow prototype defined
    in nidaqxm documentation.

    Args:
        task_idx (int): Task handle index value
        event_type (nidaqmx.constants.EveryNSamplesEventType): ACQUIRED_INTO_BUFFER
        num_samples (int): Number of samples that were read into the read buffer.
        callback_data (object)[None]: User data can be additionally passed here, if needed.
    """

    reader.read_many_sample(read_buffer, num_samples, timeout=ni.constants.WAIT_INFINITELY)

    # The callback function must return 0 to prevent raising TypeError exception.
    return 0


with ni.Task() as ao_task, ni.Task() as ai_task, ni.Task() as do_task:

    ai_args = {'min_val': -10,
            'max_val': 10,
            'terminal_config': ni.constants.TerminalConfiguration.RSE}

    ai_task.ai_channels.add_ai_voltage_chan(dev_name+ai0, **ai_args)
    ai_task.ai_channels.add_ai_voltage_chan(dev_name+ai1, **ai_args)
    ai_task.timing.cfg_samp_clk_timing(rate=fs, sample_mode=ni.constants.AcquisitionType.CONTINUOUS)
    # Configure ai to start only once ao is triggered for simultaneous generation and acquisition:
    ai_task.triggers.start_trigger.cfg_dig_edge_start_trig("ao/StartTrigger", trigger_edge=ni.constants.Edge.RISING)

    ai_task.in_stream.input_buf_size = samples_per_frame * frames_per_buffer * NR_OF_CHANNELS

    ao_args = {'min_val': -10,
               'max_val': 10}

    ao_task.ao_channels.add_ao_voltage_chan(dev_name+ao0, **ao_args)
    ao_task.ao_channels.add_ao_voltage_chan(dev_name+ao1, **ao_args)
    ao_task.timing.cfg_samp_clk_timing(rate=fs, sample_mode=ni.constants.AcquisitionType.CONTINUOUS)

    # For some reason read_buffer size is not calculating correctly based on the amount of data we preload the output
    # with (perhaps because below we fill the read_buffer using a for loop and not in one go?) so we must call out
    # explicitly:
    ao_task.out_stream.output_buf_size = samples_per_frame * frames_per_buffer * NR_OF_CHANNELS

    # ao_task.out_stream.regen_mode = ni.constants.RegenerationMode.DONT_ALLOW_REGENERATION
    # ao_task.timing.implicit_underflow_behavior = ni.constants.UnderflowBehavior.AUSE_UNTIL_DATA_AVAILABLE # SIC!
    # Typo in the package.
    #
    # NOTE: DONT_ALLOW_REGENERATION prevents repeating of previous frame on output read_buffer underrun so instead of
    # a warning the script will crash. Its good to block the regeneration during development to ensure we don't get
    # fooled by this behaviour (the warning on regeneration occurrence alone is confusing if you don't know that
    # that's the default behaviour). Additionally some NI devices will allow you to pAUSE generation till output
    # read_buffer data is available again instead of crashing (not supported by my NI6211 though).

    reader = stream_readers.AnalogMultiChannelReader(ai_task.in_stream)
    writer = stream_writers.AnalogMultiChannelWriter(ao_task.out_stream)

    terminal_name = get_terminal_name_with_dev_prefix(ao_task, "ao/SampleClock")

    # DIGITAL PULSE FOR EXTERNAL TRIGGERING
    do_task.do_channels.add_do_chan(dev_name + '/port0/line0',
        line_grouping=LineGrouping.CHAN_PER_LINE)
    do_task.write(False)

    # fill output read_buffer with data, this should also enable buffered mode
    output_frame_generator = sine_generator()
    for _ in range(frames_per_buffer):
        writer.write_many_sample(next(output_frame_generator), timeout=1)

    ai_task.register_every_n_samples_acquired_into_buffer_event(samples_per_frame, reading_task_callback)
    ao_task.register_every_n_samples_transferred_from_buffer_event(
        samples_per_frame, lambda *args: writing_task_callback(*args[:-1], output_frame_generator))
    # NOTE: The lambda function is used is to smuggle output_frame_generator instance into the writing_task_callback(
    # ) scope, under the callback_data argument. The reason to pass the generator in is to avoid the necessity to
    # define globals in the writing_task_callback() to keep track of subsequent data frames generation.

    input("Press Enter to start...")

    ai_task.start()  # arms ai but does not trigger
    ao_task.start()  # triggers both ao and ai simultaneously
    do_task.write(True)

    time.sleep(0.01)      # 10ms pulse
    do_task.write(False)  # Set LOW

    # Wait input for enter
    input("Press Enter to stop...")

    ai_task.stop()
    ao_task.stop()
    do_task.write(False)
    do_task.stop()