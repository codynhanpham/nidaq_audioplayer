"""
NiDaq Audio Playback Package

This package provides classes for NI-DAQ audio playback with advanced features:
- Singleton NI-DAQ player with thread-safe operations
- Gapless audio queue management
- Crossfading transitions between tracks
- Support for different sample rates
"""

from .player import NiDaqPlayer, get_player
from .buffer_manager import AudioBufferManager

__version__ = "1.1.0"
__author__ = "Your Name"

__all__ = [
    'NiDaqPlayer',
    'get_player', 
    'AudioBufferManager'
]
