"""
Enhanced Audio Buffer Manager for Gapless Playback

This module provides advanced buffering capabilities including:
- Crossfading between tracks
- Smart sample rate transition handling
- Pre-buffering of next tracks
- Seamless audio generation chaining
"""

import numpy as np
import soundfile as sf
from typing import Generator, Optional, Tuple, Dict, Any
import threading
from pathlib import Path


class AudioBufferError(Exception):
    """Custom exception for AudioBufferManager errors."""
    pass


class AudioBufferManager:
    """
    Advanced audio buffer manager for gapless playback.
    
    Handles:
    - Multiple audio generators for seamless transitions
    - Crossfading between tracks with different sample rates
    - Pre-buffering and smart loading
    """
    
    def __init__(self, 
                samples_per_frame: int = 8192,
                nr_of_channels: int = 2,
                voltage_scale: float = 0.1,
                crossfade_samples: int = 4096,
                flip_lr_stereo: bool = False):
        """
        Initialize buffer manager.
        
        Args:
            samples_per_frame: Samples per audio frame
            nr_of_channels: Number of output channels
            voltage_scale: Voltage scaling factor
            crossfade_samples: Number of samples for crossfade transitions
            flip_lr_stereo: Whether to flip left/right channels for stereo audio
        """
        self.samples_per_frame = samples_per_frame
        self.nr_of_channels = nr_of_channels
        self.voltage_scale = voltage_scale
        self.crossfade_samples = crossfade_samples
        self.flip_lr_stereo = flip_lr_stereo
        
        # Current and next generators
        self._current_generator: Optional[Generator] = None
        self._next_generator: Optional[Generator] = None
        self._crossfade_generator: Optional[Generator] = None
        
        # Buffer state
        self._current_file: Optional[str] = None
        self._next_file: Optional[str] = None
        self._current_sample_rate: Optional[int] = None
        self._next_sample_rate: Optional[int] = None
        
        # Crossfade state
        self._in_crossfade = False
        self._crossfade_position = 0
        self._crossfade_total = 0
        
        # Statistics
        self.stats = {
            'buffers_generated': 0,
            'crossfades_performed': 0,
            'rate_transitions': 0,
            'seamless_transitions': 0
        }
        
        # Threading for pre-loading
        self._preload_lock = threading.Lock()
    
    def update_voltage_scale(self, voltage_scale: float) -> None:
        """
        Update voltage scale for dynamic adjustment during playback.
        
        Args:
            voltage_scale: New voltage scaling factor
        """
        if voltage_scale < 0:
            raise AudioBufferError("Voltage scale must be non-negative")
        
        self.voltage_scale = voltage_scale
    
    def set_flip_lr_stereo(self, flip_lr_stereo: bool) -> None:
        """
        Update flip L/R stereo setting for dynamic adjustment during playback.
        
        Args:
            flip_lr_stereo: Whether to flip left/right channels for stereo audio
        """
        self.flip_lr_stereo = flip_lr_stereo
    
    def load_current(self, file_path: str, start_sample: int = 0) -> Dict[str, Any]:
        """
        Load current audio file.
        
        Args:
            file_path: Path to audio file
            start_sample: Starting sample position
            
        Returns:
            Audio file information
        """
        try:
            # Create generator
            self._current_generator = self._create_audio_generator(file_path, start_sample)
            self._current_file = file_path
            
            # Get file info
            info = sf.info(file_path)
            self._current_sample_rate = int(info.samplerate)
            
            return {
                'file': file_path,
                'sample_rate': self._current_sample_rate,
                'duration': info.duration,
                'channels': info.channels,
                'frames': info.frames
            }
            
        except Exception as e:
            raise AudioBufferError(f"Failed to load current audio: {e}")
    
    def preload_next(self, file_path: str) -> bool:
        """
        Preload next audio file for gapless transition.
        
        Args:
            file_path: Path to next audio file
            
        Returns:
            True if preloaded successfully
        """
        with self._preload_lock:
            try:
                # Get next file info
                info = sf.info(file_path)
                next_sample_rate = int(info.samplerate)
                
                # Create next generator
                self._next_generator = self._create_audio_generator(file_path, 0)
                self._next_file = file_path
                self._next_sample_rate = next_sample_rate
                
                return True
                
            except Exception as e:
                print(f"Failed to preload next audio: {e}")
                self._next_generator = None
                self._next_file = None
                self._next_sample_rate = None
                return False
    
    def get_next_buffer(self) -> Optional[np.ndarray]:
        """
        Get next audio buffer with automatic transitions.
        
        Returns:
            Audio buffer array or None if no more data
        """
        try:
            self.stats['buffers_generated'] += 1
            
            # Handle crossfade
            if self._in_crossfade:
                return self._get_crossfade_buffer()
            
            # Normal buffer from current generator
            if self._current_generator:
                try:
                    return next(self._current_generator)
                except StopIteration:
                    # Current track finished, try to transition
                    return self._handle_track_completion()
            
            return None
            
        except Exception as e:
            print(f"Error getting next buffer: {e}")
            return None
    
    def start_crossfade_transition(self, crossfade_samples: Optional[int] = None):
        """
        Start crossfade transition to next track.
        
        Args:
            crossfade_samples: Override default crossfade length
        """
        if not self._next_generator:
            return False
        
        with self._preload_lock:
            # Set up crossfade
            self._crossfade_total = crossfade_samples or self.crossfade_samples
            self._crossfade_position = 0
            self._in_crossfade = True
            
            self.stats['crossfades_performed'] += 1
            
            # Check for sample rate transition
            if self._current_sample_rate != self._next_sample_rate:
                self.stats['rate_transitions'] += 1
            else:
                self.stats['seamless_transitions'] += 1
            
            return True
    
    def force_transition_to_next(self):
        """Force immediate transition to next track."""
        with self._preload_lock:
            if self._next_generator:
                # Swap generators
                self._current_generator = self._next_generator
                self._current_file = self._next_file
                self._current_sample_rate = self._next_sample_rate
                
                # Clear next
                self._next_generator = None
                self._next_file = None
                self._next_sample_rate = None
                
                # Reset crossfade state
                self._in_crossfade = False
                self._crossfade_position = 0
    
    def requires_reconfiguration(self) -> bool:
        """
        Check if DAQ reconfiguration is needed for next track.
        
        Returns:
            True if sample rate change requires reconfiguration
        """
        if not self._next_sample_rate:
            return False
        
        return self._current_sample_rate != self._next_sample_rate
    
    def get_transition_info(self) -> Dict[str, Any]:
        """
        Get information about upcoming transition.
        
        Returns:
            Transition information dictionary
        """
        return {
            'has_next': self._next_generator is not None,
            'next_file': self._next_file,
            'next_sample_rate': self._next_sample_rate,
            'current_sample_rate': self._current_sample_rate,
            'requires_reconfig': self.requires_reconfiguration(),
            'in_crossfade': self._in_crossfade,
            'crossfade_progress': self._crossfade_position / max(1, self._crossfade_total),
            'stats': self.stats.copy()
        }
    
    def _create_audio_generator(self, file_path: str, start_sample: int = 0) -> Generator:
        """Create audio data generator for the specified file."""
        try:
            info = sf.info(file_path)
            total_samples = info.frames
            file_channels = info.channels
            
            # Ensure start position is valid
            current_pos = max(0, min(start_sample, total_samples - 1))
            
            while current_pos < total_samples:
                # Calculate how many samples to read for this chunk
                # Only load the necessary chunk for speed + memory usage
                chunk_samples = min(self.samples_per_frame, total_samples - current_pos)
                
                chunk_data, sample_rate = sf.read(
                    file_path, 
                    start=current_pos, 
                    frames=chunk_samples,
                    dtype='float64'
                )
                
                # Handle different channel configurations
                if chunk_data.ndim == 1:
                    # Mono - duplicate to match the number of output channels
                    if file_channels == 1:
                        # Single channel, tile to match output channels
                        chunk_data = np.tile(chunk_data, (self.nr_of_channels, 1))
                    else:
                        # This shouldn't happen, but handle gracefully
                        chunk_data = chunk_data.reshape(-1, 1).T
                        chunk_data = np.tile(chunk_data, (self.nr_of_channels, 1))
                else:
                    # Multi-channel audio
                    if chunk_data.shape[1] > self.nr_of_channels:
                        # More channels than needed - take first nr_of_channels
                        chunk_data = chunk_data[:, :self.nr_of_channels].T
                    elif chunk_data.shape[1] < self.nr_of_channels:
                        # Fewer channels than needed - duplicate to fill
                        chunk_data = chunk_data.T
                        while chunk_data.shape[0] < self.nr_of_channels:
                            chunk_data = np.vstack([chunk_data, chunk_data[-1]])
                        chunk_data = chunk_data[:self.nr_of_channels]
                    else:
                        # Exact match
                        chunk_data = chunk_data.T
                
                # Create output frame with proper size
                data_frame = np.zeros((self.nr_of_channels, self.samples_per_frame), dtype=np.float64)
                data_frame[:, :chunk_samples] = chunk_data
                
                # Apply L/R stereo channel flipping if enabled and file has exactly 2 channels
                if self.flip_lr_stereo and file_channels == 2 and self.nr_of_channels >= 2:
                    # Swap channels 0 and 1 (left and right)
                    data_frame[[0, 1]] = data_frame[[1, 0]]
                
                # Apply voltage scaling
                data_frame = data_frame * self.voltage_scale
                
                yield data_frame
                current_pos += chunk_samples
            
            return
            
        except Exception as e:
            raise AudioBufferError(f"Error creating audio generator: {e}")
    
    def _get_crossfade_buffer(self) -> Optional[np.ndarray]:
        """Generate crossfaded audio buffer."""
        try:
            # Get buffers from both generators
            current_buffer = None
            next_buffer = None
            
            if self._current_generator:
                try:
                    current_buffer = next(self._current_generator)
                except StopIteration:
                    current_buffer = np.zeros((self.nr_of_channels, self.samples_per_frame), dtype=np.float64)
            
            if self._next_generator:
                try:
                    next_buffer = next(self._next_generator)
                except StopIteration:
                    next_buffer = np.zeros((self.nr_of_channels, self.samples_per_frame), dtype=np.float64)
            
            # Handle sample rate mismatches
            if (self._current_sample_rate != self._next_sample_rate and 
                current_buffer is not None and next_buffer is not None):
                # For different sample rates, we need to resample or handle in hardware
                # For now, we'll do a quick transition rather than trying to crossfade
                crossfade_buffer = self._create_quick_transition(current_buffer, next_buffer)
            else:
                # Same sample rate - normal crossfade
                crossfade_buffer = self._create_crossfade(current_buffer, next_buffer)
            
            # Update crossfade position
            self._crossfade_position += self.samples_per_frame
            
            # Check if crossfade is complete
            if self._crossfade_position >= self._crossfade_total:
                self._complete_crossfade()
            
            return crossfade_buffer
            
        except Exception as e:
            print(f"Error creating crossfade buffer: {e}")
            self._complete_crossfade()  # Emergency exit from crossfade
            return self._get_silence_buffer()
    
    def _create_crossfade(self, 
                        current_buffer: Optional[np.ndarray], 
                        next_buffer: Optional[np.ndarray]) -> np.ndarray:
        """Create crossfaded buffer from two source buffers."""
        if current_buffer is None and next_buffer is None:
            return self._get_silence_buffer()
        
        if current_buffer is None:
            return next_buffer
        
        if next_buffer is None:
            return current_buffer
        
        # Calculate crossfade weights
        samples_remaining = self._crossfade_total - self._crossfade_position
        fade_samples = min(self.samples_per_frame, samples_remaining)
        
        # Create weight arrays
        current_weight = np.linspace(1.0, 0.0, fade_samples)
        next_weight = np.linspace(0.0, 1.0, fade_samples)
        
        # Apply crossfade
        result = current_buffer.copy()
        
        for channel in range(self.nr_of_channels):
            result[channel, :fade_samples] = (
                current_buffer[channel, :fade_samples] * current_weight +
                next_buffer[channel, :fade_samples] * next_weight
            )
            
            # Fill remainder with next buffer if crossfade extends beyond current frame
            if fade_samples < self.samples_per_frame:
                result[channel, fade_samples:] = next_buffer[channel, fade_samples:]
        
        return result
    
    def _create_quick_transition(self, 
                                current_buffer: Optional[np.ndarray], 
                                next_buffer: Optional[np.ndarray]) -> np.ndarray:
        """
        Create quick transition for sample rate changes.
        
        When sample rates differ, we can't crossfade in the buffer domain.
        Instead, do a very short fade to minimize clicks.
        """
        if next_buffer is None:
            return current_buffer if current_buffer is not None else self._get_silence_buffer()
        
        if current_buffer is None:
            return next_buffer
        
        # Very short fade (e.g., 64 samples) to minimize clicks
        quick_fade_samples = min(64, self.samples_per_frame // 4)
        
        result = next_buffer.copy()
        
        if quick_fade_samples > 0:
            fade_in = np.linspace(0.0, 1.0, quick_fade_samples)
            for channel in range(self.nr_of_channels):
                result[channel, :quick_fade_samples] *= fade_in
        
        return result
    
    def _complete_crossfade(self):
        """Complete crossfade transition."""
        with self._preload_lock:
            # Swap to next generator
            self._current_generator = self._next_generator
            self._current_file = self._next_file
            self._current_sample_rate = self._next_sample_rate
            
            # Clear next
            self._next_generator = None
            self._next_file = None
            self._next_sample_rate = None
            
            # Reset crossfade state
            self._in_crossfade = False
            self._crossfade_position = 0
    
    def _handle_track_completion(self) -> Optional[np.ndarray]:
        """Handle completion of current track."""
        if self._next_generator:
            # Automatic transition
            self.force_transition_to_next()
            return self.get_next_buffer()
        else:
            # No next track, return silence
            return None
    
    def _get_silence_buffer(self) -> np.ndarray:
        """Get a buffer filled with silence."""
        return np.zeros((self.nr_of_channels, self.samples_per_frame), dtype=np.float64)
    
    def cleanup(self):
        """Clean up resources."""
        self._current_generator = None
        self._next_generator = None
        self._crossfade_generator = None
        self._in_crossfade = False
