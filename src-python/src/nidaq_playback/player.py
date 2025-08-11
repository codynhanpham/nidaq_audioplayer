"""
NiDaq Audio Player - Singleton Class for NI-DAQ Audio Playback

This module provides a singleton NiDaqPlayer class that handles:
- Device configuration and channel setup
- Audio file loading and buffer management
- Playback control (start/stop/pause/seek)
- Thread-safe singleton pattern to prevent multiple instances accessing the same device
"""

import time
import threading
import numpy as np
import soundfile as sf
from typing import List, Optional, Dict, Any, Generator
from pathlib import Path
import weakref
import atexit

import nidaqmx as ni
from nidaqmx import stream_readers, stream_writers
from nidaqmx.constants import AcquisitionType, LineGrouping, ProductCategory, TerminalConfiguration, Edge

from .buffer_manager import AudioBufferManager


class NiDaqPlayerError(Exception):
    """Custom exception for NiDaqPlayer errors."""
    pass


class NiDaqPlayer:
    """
    Singleton NI-DAQ Audio Player class.
    
    Handles audio playback through NI-DAQ devices with configurable channels.
    Ensures only one instance can access a device at a time.
    """
    
    _instances: Dict[str, weakref.ReferenceType] = {}
    _lock = threading.Lock()
    
    def __new__(cls, device_name: str = 'Dev1', **kwargs):
        """Create singleton instance per device."""
        with cls._lock:
            # Check if instance exists for this device
            if device_name in cls._instances:
                existing = cls._instances[device_name]()
                if existing is not None:
                    # Stop and cleanup existing instance
                    existing.stop()
                    existing._cleanup()
            
            # Create new instance
            instance = super().__new__(cls)
            cls._instances[device_name] = weakref.ref(instance, cls._cleanup_weakref)
            return instance
    
    @classmethod
    def _cleanup_weakref(cls, ref):
        """Clean up weakref when instance is garbage collected."""
        with cls._lock:
            # Find and remove the weakref
            to_remove = []
            for device_name, weak_ref in cls._instances.items():
                if weak_ref is ref:
                    to_remove.append(device_name)
            for device_name in to_remove:
                del cls._instances[device_name]
    
    def __init__(self, 
        device_name: str = 'Dev1',
        ao_channels: List[str] = None,
        ai_channels: List[str] = None,
        do_channels: List[str] = None,
        sample_rate: int = 44100,
        samples_per_frame: int = 8192,
        frames_per_buffer: int = 10,
        voltage_scale: float = 0.1,
        ai_voltage_range: tuple = (-10, 10),
        ao_voltage_range: tuple = (-10, 10)):
        """
        Initialize NiDaqPlayer.
        
        Args:
            device_name: NI-DAQ device name (e.g., 'Dev1')
            ao_channels: List of analog output channels (e.g., ['/ao0', '/ao1'])
            ai_channels: List of analog input channels (e.g., ['/ai0', '/ai1'])
            do_channels: List of digital output channels (e.g., ['/port0/line0', '/port0/line1'])
            sample_rate: Default sample rate (will be overridden by audio file)
            samples_per_frame: Number of samples per frame
            frames_per_buffer: Number of frames in buffer
            voltage_scale: Scale factor for output voltage
            ai_voltage_range: Min/max voltage range for analog input
            ao_voltage_range: Min/max voltage range for analog output
        """
        # Prevent re-initialization of existing instance
        if hasattr(self, '_initialized'):
            return
        
        # Default channel configurations
        if ao_channels is None:
            ao_channels = ['/ao0', '/ao1']
        if ai_channels is None:
            ai_channels = ['/ai0', '/ai1']
        if do_channels is None:
            do_channels = ['/port0/line0', '/port0/line1']
        
        # Validate configuration
        if len(ao_channels) != len(ai_channels):
            raise NiDaqPlayerError("Number of analog output channels must match analog input channels")
        if len(ao_channels) == 0:
            raise NiDaqPlayerError("At least one analog output channel must be specified")
        
        # Store configuration
        self.device_name = device_name
        self.ao_channels = ao_channels
        self.ai_channels = ai_channels
        self.do_channels = do_channels
        self.nr_of_channels = len(ao_channels)
        self.sample_rate = sample_rate
        self.samples_per_frame = samples_per_frame
        self.frames_per_buffer = frames_per_buffer
        self.voltage_scale = voltage_scale
        self.ai_voltage_range = ai_voltage_range
        self.ao_voltage_range = ao_voltage_range
        
        # State variables
        self._tasks_created = False
        self._audio_loaded = False
        self._playing = False
        self._paused = False
        self._audio_completed = False
        self._current_audio_file = None
        self._audio_duration = 0.0
        self._audio_sample_count = 0
        self._pause_position = 0  # Position in samples where playback was paused
        self._total_samples_generated = 0  # Total samples generated in current session
        
        # Task objects
        self.ao_task: Optional[ni.Task] = None
        self.ai_task: Optional[ni.Task] = None
        self.do_task: Optional[ni.Task] = None
        
        # Stream objects
        self.reader: Optional[stream_readers.AnalogMultiChannelReader] = None
        self.writer: Optional[stream_writers.AnalogMultiChannelWriter] = None
        
        # Audio data
        self._audio_generator: Optional[Generator] = None
        self._read_buffer: Optional[np.ndarray] = None
        self._buffer_manager: Optional[AudioBufferManager] = None
        
        # Enhanced playback features
        self._gapless_enabled = False
        self._crossfade_enabled = False
        self._crossfade_samples = 4096
        
        # Threading
        self._state_lock = threading.RLock()
        
        # Register cleanup on exit
        atexit.register(self._cleanup)
        
        # Initialize buffer manager
        self._buffer_manager = AudioBufferManager(
            samples_per_frame=self.samples_per_frame,
            nr_of_channels=self.nr_of_channels,
            voltage_scale=self.voltage_scale,
            crossfade_samples=self._crossfade_samples
        )
        
        self._initialized = True
        print(f"NiDaqPlayer initialized for device {device_name} with {self.nr_of_channels} channels")
    
    def configure_device(self,
                        device_name: str = None,
                        ao_channels: List[str] = None,
                        ai_channels: List[str] = None,
                        do_channels: List[str] = None) -> None:
        """
        Reconfigure device and channels.
        
        Args:
            device_name: New device name
            ao_channels: New analog output channels
            ai_channels: New analog input channels
            do_channels: New digital output channels
        """
        with self._state_lock:
            # Stop any current playback
            if self._playing:
                self.stop()
            
            # Update configuration
            if device_name is not None:
                self.device_name = device_name
            if ao_channels is not None:
                self.ao_channels = ao_channels
            if ai_channels is not None:
                self.ai_channels = ai_channels
            if do_channels is not None:
                self.do_channels = do_channels
            
            # Validate new configuration
            if len(self.ao_channels) != len(self.ai_channels):
                raise NiDaqPlayerError("Number of analog output channels must match analog input channels")
            
            self.nr_of_channels = len(self.ao_channels)
            
            # Clear existing tasks
            self._clear_tasks()
            self._audio_loaded = False
            
            print(f"Device reconfigured: {self.device_name} with {self.nr_of_channels} channels")
    
    def load_audio(self, audio_file: str) -> Dict[str, Any]:
        """
        Load audio file and prepare for playback.
        
        Args:
            audio_file: Path to audio file
            
        Returns:
            Dict with audio file information
        """
        with self._state_lock:
            audio_path = Path(audio_file)
            if not audio_path.exists():
                raise NiDaqPlayerError(f"Audio file not found: {audio_file}")
            
            try:
                # Get audio file info
                info = sf.info(audio_file)
                self.sample_rate = int(info.samplerate)
                self._audio_duration = info.duration
                self._audio_sample_count = info.frames
                self._current_audio_file = str(audio_path)
                
                # Stop any current playback
                if self._playing:
                    self.stop()
                
                # Clear existing tasks and create new ones
                self._clear_tasks()
                self._create_tasks()
                
                # Initialize buffer manager with current file
                if self._buffer_manager:
                    buffer_info = self._buffer_manager.load_current(audio_file, start_sample=0)
                    # Don't create separate audio generator - use buffer manager exclusively
                    self._audio_generator = None
                else:
                    # Fallback: Create audio generator only if no buffer manager
                    self._audio_generator = self._create_audio_generator(audio_file, start_sample=0)
                
                self._prime_buffer()
                
                self._audio_loaded = True
                self._audio_completed = False
                self._pause_position = 0
                self._total_samples_generated = 0
                
                audio_info = {
                    'file': self._current_audio_file,
                    'sample_rate': self.sample_rate,
                    'duration': self._audio_duration,
                    'channels': info.channels,
                    'frames': self._audio_sample_count
                }
                
                print(f"Audio loaded: {audio_path.name} ({self._audio_duration:.2f}s @ {self.sample_rate}Hz)")
                return audio_info
                
            except Exception as e:
                raise NiDaqPlayerError(f"Failed to load audio file: {e}")
    
    def play(self) -> None:
        """Start or resume audio playback."""
        with self._state_lock:
            if not self._audio_loaded:
                raise NiDaqPlayerError("No audio file loaded. Call load_audio() first.")
            
            if self._playing:
                print("Audio is already playing")
                return
            
            if self._audio_completed and not self._paused:
                print("Audio playback already completed")
                return
            
            try:
                # If resuming from pause, recreate tasks and generator from pause position
                if self._paused:
                    print(f"Resuming audio playback from {self._pause_position / self.sample_rate:.2f}s...")
                    self._clear_tasks()
                    self._create_tasks()
                    
                    # Use buffer manager for resume if available
                    if self._buffer_manager:
                        # Reload current file from pause position
                        self._buffer_manager.load_current(self._current_audio_file, start_sample=self._pause_position)
                        self._audio_generator = None
                    else:
                        # Fallback: Create audio generator starting from pause position
                        self._audio_generator = self._create_audio_generator(
                            self._current_audio_file, start_sample=self._pause_position)
                    
                    self._prime_buffer()
                    
                    # Reset total samples counter for this session
                    self._total_samples_generated = 0
                else:
                    print("Starting audio playback...")
                
                # Start tasks
                self.ai_task.start()  # Arms AI but doesn't trigger
                # Send digital trigger pulse asynchronously
                self.do_task.write([True] * len(self.do_channels))
                self.ao_task.start()  # Triggers both AO and AI simultaneously
                
                self._playing = True
                self._paused = False
                
                print("Audio playback resumed" if self._pause_position > 0 else "Audio playback started")
                
            except Exception as e:
                self._playing = False
                raise NiDaqPlayerError(f"Failed to start playback: {e}")
    
    def stop(self) -> None:
        """Stop audio playback and reset to beginning."""
        with self._state_lock:
            if not self._playing and not self._paused:
                return
            
            try:
                print("Stopping audio playback...")
                
                # Stop tasks
                if self.ai_task:
                    self.ai_task.stop()
                if self.ao_task:
                    self.ao_task.stop()
                if self.do_task:
                    self.do_task.write([False] * len(self.do_channels))
                    self.do_task.stop()
                
                self._playing = False
                self._paused = False
                self._pause_position = 0  # Reset position
                self._total_samples_generated = 0
                
                print("Audio playback stopped and reset")
                
            except Exception as e:
                print(f"Error stopping playback: {e}")
            finally:
                self._playing = False
                self._paused = False
    
    def pause(self) -> None:
        """Pause audio playback, preserving position for resume."""
        with self._state_lock:
            if not self._playing:
                raise NiDaqPlayerError("No audio currently playing")
            
            if self._paused:
                print("Audio is already paused")
                return
            
            try:
                print("Pausing audio playback...")
                
                # Get current position before stopping
                if self.ao_task:
                    current_samples = self.ao_task.out_stream.total_samp_per_chan_generated
                    self._pause_position += current_samples - self._total_samples_generated
                    self._total_samples_generated = current_samples
                
                # Stop tasks but don't reset position
                if self.ai_task:
                    self.ai_task.stop()
                if self.ao_task:
                    self.ao_task.stop()
                if self.do_task:
                    self.do_task.write([False] * len(self.do_channels))
                    self.do_task.stop()
                
                self._playing = False
                self._paused = True
                
                pause_time = self._pause_position / self.sample_rate
                print(f"Audio playback paused at {pause_time:.2f}s")
                
            except Exception as e:
                print(f"Error pausing playback: {e}")
                self._playing = False
    
    def resume(self) -> None:
        """Resume audio playback from paused position."""
        with self._state_lock:
            if not self._paused:
                if self._playing:
                    print("Audio is already playing")
                    return
                else:
                    raise NiDaqPlayerError("No audio currently paused. Use play() to start playback.")
            
            if self._audio_completed:
                print("Audio playback already completed")
                return
            
            # Resume is handled by play() method
            self.play()
    
    def seek(self, position: float) -> None:
        """
        Seek to a specific position in the audio file.
        
        Args:
            position: Time position in seconds to seek to
        """
        with self._state_lock:
            if not self._audio_loaded:
                raise NiDaqPlayerError("No audio file loaded. Call load_audio() first.")
            
            if position < 0:
                position = 0
            elif position > self._audio_duration:
                position = self._audio_duration
            
            # Convert time to sample position
            seek_sample = int(position * self.sample_rate)
            if seek_sample >= self._audio_sample_count:
                seek_sample = self._audio_sample_count - 1
            
            was_playing = self._playing
            
            try:
                # Stop current playback if running
                if self._playing:
                    self.stop()
                
                # Update pause position to the seek position
                self._pause_position = seek_sample
                self._total_samples_generated = 0
                self._audio_completed = False
                
                # If we were playing, restart from new position
                if was_playing:
                    # Set as paused so play() will resume from the new position
                    self._paused = True
                    self.play()
                else:
                    # Just set as paused at the new position
                    self._paused = True
                
                seek_time = self._pause_position / self.sample_rate
                print(f"Seeked to {seek_time:.2f}s (sample {self._pause_position})")
                
            except Exception as e:
                raise NiDaqPlayerError(f"Failed to seek to position {position}s: {e}")
    
    def get_status(self) -> Dict[str, Any]:
        """Get current player status."""
        with self._state_lock:
            status = {
                'device_name': self.device_name,
                'ao_channels': self.ao_channels,
                'ai_channels': self.ai_channels,
                'do_channels': self.do_channels,
                'sample_rate': self.sample_rate,
                'audio_loaded': self._audio_loaded,
                'playing': self._playing,
                'paused': self._paused,
                'current_file': self._current_audio_file,
                'duration': self._audio_duration,
                'nr_of_channels': self.nr_of_channels,
                'pause_position': self._pause_position,
                'total_audio_samples': self._audio_sample_count
            }
            
            # Add playback position if tasks are active
            if self._playing and self.ao_task:
                try:
                    current_samples = self.ao_task.out_stream.total_samp_per_chan_generated
                    total_position = self._pause_position + (current_samples - self._total_samples_generated)
                    current_time = total_position / self.sample_rate if self.sample_rate > 0 else 0
                    status['current_time'] = current_time
                    status['samples_generated'] = total_position
                    status['session_samples'] = current_samples
                    
                    # Check if all audio samples have been actually generated by DAQ
                    # This is the proper way to determine if playback is complete
                    status['audio_completed'] = total_position >= self._audio_sample_count
                except:
                    status['current_time'] = self._pause_position / self.sample_rate if self.sample_rate > 0 else 0
                    status['samples_generated'] = self._pause_position
                    status['session_samples'] = 0
                    status['audio_completed'] = self._audio_completed
            elif self._paused:
                # Show paused position
                status['current_time'] = self._pause_position / self.sample_rate if self.sample_rate > 0 else 0
                status['samples_generated'] = self._pause_position
                status['session_samples'] = 0
                status['audio_completed'] = self._pause_position >= self._audio_sample_count
            else:
                status['current_time'] = 0
                status['samples_generated'] = 0
                status['session_samples'] = 0
                status['audio_completed'] = self._audio_completed
            
            return status
    
    def _create_tasks(self) -> None:
        """Create NI-DAQ tasks."""
        if self._tasks_created:
            return
        
        try:
            # Create tasks
            self.ao_task = ni.Task()
            self.ai_task = ni.Task()
            self.do_task = ni.Task()
            
            # Configure analog input
            ai_args = {
                'min_val': self.ai_voltage_range[0],
                'max_val': self.ai_voltage_range[1],
                'terminal_config': TerminalConfiguration.RSE
            }
            
            for ai_channel in self.ai_channels:
                self.ai_task.ai_channels.add_ai_voltage_chan(
                    self.device_name + ai_channel, **ai_args)
            
            self.ai_task.timing.cfg_samp_clk_timing(
                rate=self.sample_rate, 
                sample_mode=AcquisitionType.CONTINUOUS)
            self.ai_task.triggers.start_trigger.cfg_dig_edge_start_trig(
                "ao/StartTrigger", trigger_edge=Edge.RISING)
            self.ai_task.in_stream.input_buf_size = (
                self.samples_per_frame * self.frames_per_buffer * self.nr_of_channels)
            
            # Configure analog output
            ao_args = {
                'min_val': self.ao_voltage_range[0],
                'max_val': self.ao_voltage_range[1]
            }
            
            for ao_channel in self.ao_channels:
                self.ao_task.ao_channels.add_ao_voltage_chan(
                    self.device_name + ao_channel, **ao_args)
            
            self.ao_task.timing.cfg_samp_clk_timing(
                rate=self.sample_rate,
                sample_mode=AcquisitionType.CONTINUOUS)
            self.ao_task.out_stream.output_buf_size = (
                self.samples_per_frame * self.frames_per_buffer * self.nr_of_channels)
            
            # Configure digital output
            for do_channel in self.do_channels:
                self.do_task.do_channels.add_do_chan(
                    self.device_name + do_channel,
                    line_grouping=LineGrouping.CHAN_PER_LINE)
            self.do_task.write([False] * len(self.do_channels))
            
            # Create stream readers/writers
            self.reader = stream_readers.AnalogMultiChannelReader(self.ai_task.in_stream)
            self.writer = stream_writers.AnalogMultiChannelWriter(self.ao_task.out_stream)
            
            # Create read buffer
            self._read_buffer = np.zeros((self.nr_of_channels, self.samples_per_frame), dtype=np.float64)
            
            # Register callbacks
            self.ai_task.register_every_n_samples_acquired_into_buffer_event(
                self.samples_per_frame, self._reading_callback)
            self.ao_task.register_every_n_samples_transferred_from_buffer_event(
                self.samples_per_frame, self._writing_callback)
            self.ao_task.register_done_event(self._done_callback)
            
            self._tasks_created = True
            
        except Exception as e:
            self._clear_tasks()
            raise NiDaqPlayerError(f"Failed to create tasks: {e}")
    
    def _clear_tasks(self) -> None:
        """Clear and close all NI-DAQ tasks."""
        try:
            if self.ao_task:
                try:
                    self.ao_task.close()
                except:
                    pass
                self.ao_task = None
            
            if self.ai_task:
                try:
                    self.ai_task.close()
                except:
                    pass
                self.ai_task = None
            
            if self.do_task:
                try:
                    self.do_task.close()
                except:
                    pass
                self.do_task = None
            
            self.reader = None
            self.writer = None
            self._read_buffer = None
            self._tasks_created = False
            
        except Exception as e:
            print(f"Error clearing tasks: {e}")
    
    def _create_audio_generator(self, file_path: str, start_sample: int = 0) -> Generator:
        """Create audio data generator starting from specified sample position."""
        try:
            data, sample_rate = sf.read(file_path, dtype='float64')
            
            # Handle different channel configurations
            if data.ndim == 1:
                # Mono - duplicate to match the number of output channels
                data = np.tile(data, (self.nr_of_channels, 1))
            else:
                # Multi-channel audio
                if data.shape[1] > self.nr_of_channels:
                    # More channels than needed - take first nr_of_channels
                    data = data[:, :self.nr_of_channels].T
                elif data.shape[1] < self.nr_of_channels:
                    # Fewer channels than needed - duplicate to fill
                    data = data.T
                    while data.shape[0] < self.nr_of_channels:
                        data = np.vstack([data, data[-1]])
                    data = data[:self.nr_of_channels]
                else:
                    # Exact match
                    data = data.T
            
            # Scale to voltage range
            data = data * self.voltage_scale
            
            # Yield chunks of the specified size starting from start_sample
            total_samples = data.shape[1]
            current_pos = start_sample
            
            # Ensure start position is valid
            if current_pos >= total_samples:
                print(f"Warning: Start position {current_pos} is beyond audio length {total_samples}")
                current_pos = total_samples - 1
            
            while current_pos < total_samples:
                end_pos = min(current_pos + self.samples_per_frame, total_samples)
                chunk_size = end_pos - current_pos
                
                data_frame = np.zeros((self.nr_of_channels, self.samples_per_frame), dtype=np.float64)
                data_frame[:, :chunk_size] = data[:, current_pos:end_pos]
                
                yield data_frame
                current_pos = end_pos
            
            return
            
        except Exception as e:
            raise NiDaqPlayerError(f"Error creating audio generator: {e}")
    
    def _prime_buffer(self) -> None:
        """Prime the output buffer with initial audio data."""
        try:
            for _ in range(self.frames_per_buffer):
                # Use buffer manager if available, fallback to generator
                if self._buffer_manager:
                    buffer = self._buffer_manager.get_next_buffer()
                    if buffer is not None:
                        self.writer.write_many_sample(buffer, timeout=1.0)
                    else:
                        raise NiDaqPlayerError("Buffer manager returned no data for priming")
                elif self._audio_generator:
                    self.writer.write_many_sample(next(self._audio_generator), timeout=1.0)
                else:
                    raise NiDaqPlayerError("No audio generator or buffer manager available")
        except StopIteration:
            raise NiDaqPlayerError("Audio file too short to prime buffer")
        except Exception as e:
            raise NiDaqPlayerError(f"Failed to prime buffer: {e}")
    
    def _writing_callback(self, task_idx, event_type, num_samples, callback_data=None):
        """Callback for writing audio data."""
        try:
            # Check for completion based on actual samples generated
            if self._check_playback_completion():
                # All audio has been generated, we can stop writing new data
                return 0
            
            if self._buffer_manager:
                # Use enhanced buffer manager
                buffer = self._buffer_manager.get_next_buffer()
                if buffer is not None:
                    self.writer.write_many_sample(buffer, timeout=10.0)
                else:
                    # No more audio data, but keep writing silence until DAQ finishes
                    # The completion will be handled by _check_playback_completion()
                    silence = np.zeros((self.nr_of_channels, self.samples_per_frame), dtype=np.float64)
                    self.writer.write_many_sample(silence, timeout=10.0)
            elif self._audio_generator:
                # Fallback to original generator
                try:
                    self.writer.write_many_sample(
                        next(self._audio_generator), timeout=10.0)
                except StopIteration:
                    # Audio generator finished, write silence but don't mark as completed yet
                    silence = np.zeros((self.nr_of_channels, self.samples_per_frame), dtype=np.float64)
                    self.writer.write_many_sample(silence, timeout=10.0)
        except Exception as e:
            print(f"Writing callback error: {e}")
        
        return 0
    
    def _reading_callback(self, task_idx, event_type, num_samples, callback_data=None):
        """Callback for reading input data."""
        try:
            if self._read_buffer is not None:
                self.reader.read_many_sample(
                    self._read_buffer, num_samples, 
                    timeout=ni.constants.WAIT_INFINITELY)
        except Exception as e:
            print(f"Reading callback error: {e}")
        
        return 0
    
    def _done_callback(self, task_idx, status, callback_data=None):
        """Callback when audio playback is done."""
        print("Audio playback finished.")
        
        try:
            if self.do_task:
                self.do_task.write([False] * len(self.do_channels))
        except Exception as e:
            print(f"Error resetting digital output channels: {e}")
        
        self._playing = False
        self._audio_completed = True
        return 0
    
    def _check_playback_completion(self) -> bool:
        """
        Check if playback is actually complete based on samples generated.
        
        Returns:
            True if all audio samples have been generated by the DAQ
        """
        if not self._playing or not self.ao_task:
            return self._audio_completed
        
        try:
            current_samples = self.ao_task.out_stream.total_samp_per_chan_generated
            total_position = self._pause_position + (current_samples - self._total_samples_generated)
            
            # Playback is complete when we've generated all the audio samples
            if total_position >= self._audio_sample_count:
                if not self._audio_completed:
                    print(f"Audio playback completed: {total_position}/{self._audio_sample_count} samples generated")
                    
                    # Reset digital output channels to False
                    try:
                        if self.do_task:
                            self.do_task.write([False] * len(self.do_channels))
                            print("Digital output channels reset to False")
                    except Exception as e:
                        print(f"Error resetting digital output channels: {e}")
                    
                    self._audio_completed = True
                    self._playing = False
                return True
            
            return False
            
        except Exception as e:
            print(f"Error checking playback completion: {e}")
            return self._audio_completed
    
    def _cleanup(self) -> None:
        """Clean up resources."""
        with self._state_lock:
            try:
                if self._playing:
                    self.stop()
                self._clear_tasks()
                if self._buffer_manager:
                    self._buffer_manager.cleanup()
                print(f"NiDaqPlayer cleanup completed for device {self.device_name}")
            except Exception as e:
                print(f"Error during cleanup: {e}")
    
    # Enhanced Gapless Playback Methods
    
    def enable_gapless_playback(self, enable: bool = True, crossfade_samples: int = 4096):
        """
        Enable or disable gapless playback with crossfading.
        
        Args:
            enable: Enable gapless playback
            crossfade_samples: Number of samples for crossfade transition
        """
        with self._state_lock:
            self._gapless_enabled = enable
            self._crossfade_enabled = enable
            self._crossfade_samples = crossfade_samples
            
            if self._buffer_manager:
                self._buffer_manager.crossfade_samples = crossfade_samples
            
            print(f"Gapless playback {'enabled' if enable else 'disabled'}")
            if enable:
                print(f"Crossfade samples: {crossfade_samples}")
    
    def preload_next_audio(self, audio_file: str) -> bool:
        """
        Preload next audio file for gapless transition.
        
        Args:
            audio_file: Path to next audio file
            
        Returns:
            True if preloaded successfully
        """
        if not self._gapless_enabled or not self._buffer_manager:
            return False
        
        try:
            audio_path = Path(audio_file)
            if not audio_path.exists():
                print(f"Audio file not found: {audio_file}")
                return False
            
            success = self._buffer_manager.preload_next(str(audio_path))
            if success:
                print(f"Preloaded next audio: {audio_path.name}")
            else:
                print(f"Failed to preload: {audio_path.name}")
            
            return success
            
        except Exception as e:
            print(f"Error preloading audio: {e}")
            return False
    
    def start_crossfade_transition(self, crossfade_samples: Optional[int] = None) -> bool:
        """
        Start crossfade transition to preloaded next track.
        
        Args:
            crossfade_samples: Override default crossfade length
            
        Returns:
            True if crossfade started successfully
        """
        if not self._gapless_enabled or not self._buffer_manager:
            return False
        
        if not self._playing:
            print("Cannot start crossfade: no audio currently playing")
            return False
        
        success = self._buffer_manager.start_crossfade_transition(crossfade_samples)
        if success:
            print("Crossfade transition started")
        else:
            print("Failed to start crossfade: no next track preloaded")
        
        return success
    
    def get_transition_status(self) -> Dict[str, Any]:
        """
        Get status of gapless transition capabilities.
        
        Returns:
            Transition status information
        """
        status = {
            'gapless_enabled': self._gapless_enabled,
            'crossfade_enabled': self._crossfade_enabled,
            'crossfade_samples': self._crossfade_samples
        }
        
        if self._buffer_manager:
            buffer_info = self._buffer_manager.get_transition_info()
            status.update(buffer_info)
        
        return status
    
    def load_and_transition_to_next(self, audio_file: str, use_crossfade: bool = True) -> bool:
        """
        Load next audio and immediately transition to it.
        
        Args:
            audio_file: Path to next audio file
            use_crossfade: Use crossfade transition if enabled
            
        Returns:
            True if transition started successfully
        """
        if not self._gapless_enabled or not self._buffer_manager:
            # Fall back to standard load_audio method
            try:
                self.load_audio(audio_file)
                if not self._playing:
                    self.play()
                return True
            except Exception as e:
                print(f"Fallback load failed: {e}")
                return False
        
        try:
            # Preload the next file
            if not self.preload_next_audio(audio_file):
                return False
            
            # Check if reconfiguration is needed
            if self._buffer_manager.requires_reconfiguration():
                print("Sample rate change detected - will reconfigure DAQ after transition")
                # For sample rate changes, we need to handle differently
                # This might require stopping and restarting with new config
                return self._handle_sample_rate_transition(audio_file)
            
            # Start crossfade if requested and enabled
            if use_crossfade and self._crossfade_enabled:
                return self.start_crossfade_transition()
            else:
                # Force immediate transition
                self._buffer_manager.force_transition_to_next()
                return True
                
        except Exception as e:
            print(f"Error in load_and_transition_to_next: {e}")
            return False
    
    def _handle_sample_rate_transition(self, audio_file: str) -> bool:
        """
        Handle transition between files with different sample rates.
        
        This requires stopping the current DAQ configuration and starting
        a new one with the new sample rate.
        
        Args:
            audio_file: Next audio file path
            
        Returns:
            True if transition successful
        """
        try:
            # Get current playback position for reference
            current_status = self.get_status()
            
            print("Handling sample rate transition...")
            
            # Stop current playback
            self.stop()
            
            # Load new audio (this will reconfigure DAQ)
            audio_info = self.load_audio(audio_file)
            print(f"Transitioned to new sample rate: {audio_info['sample_rate']}Hz")
            
            # Resume playback
            self.play()
            
            return True
            
        except Exception as e:
            print(f"Error handling sample rate transition: {e}")
            return False
    
    def __del__(self):
        """Destructor to ensure cleanup."""
        self._cleanup()
    
    def __enter__(self):
        """Context manager entry."""
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit."""
        self._cleanup()


# Convenience function to get or create player instance
def get_player(device_name: str = 'Dev1', **kwargs) -> NiDaqPlayer:
    """
    Get or create NiDaqPlayer instance for the specified device.
    
    Args:
        device_name: NI-DAQ device name
        **kwargs: Additional configuration parameters
        
    Returns:
        NiDaqPlayer instance
    """
    return NiDaqPlayer(device_name=device_name, **kwargs)
