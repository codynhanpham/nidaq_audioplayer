import os
import time
from typing import Dict, Any, Union
from nidaq_playback import player as Player
from ..utils import create_success_response, create_error_response, validate_required_fields


# Global Variable for current NIDAQ Player object
nidaq_player: Union[None, Player.NiDaqPlayer] = None


async def handle_load_audio(websocket, data: Any) -> Dict[str, Any]:
    """Handle load_audio task - loads an audio file for playback."""
    global nidaq_player

    try:
        # Validate required fields
        is_valid, error_msg = validate_required_fields(data, ["file_path", "device_name", "ao_channels"])
        if not is_valid:
            return create_error_response(error_msg, task_name="load_audio")

        file_path = data["file_path"]
        if not os.path.exists(file_path):
            return create_error_response(f"Audio file not found: {file_path}", task_name="load_audio")

        supported_formats = ['.wav', '.mp3', '.flac', '.ogg', '.m4a', '.aiff']
        file_extension = os.path.splitext(file_path)[1].lower()
        
        if file_extension not in supported_formats:
            return create_error_response(f"Unsupported audio format: {file_extension}. Supported: {supported_formats}", task_name="load_audio")

        # Extract additional parameters
        device_name = data["device_name"]
        ao_channels = data["ao_channels"]
        ai_channels = data.get("ai_channels", [])
        do_channels = data.get("do_channels", [])
        volume = data.get("volume", 20)
        volume = max(0, min(volume, 100))
        voltage_scale = volume / 100.0
        samples_per_frame = data.get("samples_per_frame", 8192)
        flip_lr_stereo = data.get("flip_lr_stereo", False)

        # Init or update nidaq_player with the selected file
        try:
            if nidaq_player is None or not isinstance(nidaq_player, Player.NiDaqPlayer):
                nidaq_player = None
                nidaq_player = Player.NiDaqPlayer(
                    device_name=device_name,
                    ao_channels=ao_channels,
                    ai_channels=ai_channels,
                    do_channels=do_channels,
                    voltage_scale=voltage_scale,
                    samples_per_frame=samples_per_frame
                )
            else:
                nidaq_player.stop()

            # Load the audio file
            nidaq_player.load_audio(file_path)
            
            # Set the flip L/R stereo configuration
            nidaq_player.set_flip_lr_stereo(flip_lr_stereo)

            status = nidaq_player.get_status()

            
            response_data = {
                "message": "Audio file loaded successfully",
                "player_info": status,
            }

            return create_success_response(response_data, task_name="load_audio")

        except Exception as e:
            return create_error_response(f"Failed to load audio file: {str(e)}", task_name="load_audio")

    except Exception as e:
        return create_error_response(f"Load audio task failed: {str(e)}", task_name="load_audio")


def get_loaded_audio_info() -> Dict[str, Any]:
    """Get information about currently loaded audio."""
    return nidaq_player.get_status()


def clear_loaded_audio():
    """Clear currently loaded audio information."""
    global nidaq_player
    if nidaq_player:
        nidaq_player.stop()
        nidaq_player = None
