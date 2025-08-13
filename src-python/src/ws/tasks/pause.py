from typing import Dict, Any
from . import load_audio
from ..utils import create_success_response, create_error_response


async def handle_pause(websocket, data: Any) -> Dict[str, Any]:
    """Handle pause task - pauses audio playback."""
    
    try:
        if load_audio.nidaq_player is None:
            return create_error_response("No audio player initialized. Load audio first.")
        
        if not load_audio.nidaq_player._playing:
            return create_error_response("No audio currently playing.")
        
        load_audio.nidaq_player.pause()
        
        status = load_audio.nidaq_player.get_status()
        
        response_data = {
            "message": "Playback paused",
            "status": status,
            "pause_position": status.get('current_time', 0)
        }
        
        return create_success_response(response_data)
        
    except Exception as e:
        return create_error_response(f"Pause task failed: {str(e)}")


async def handle_resume(websocket, data: Any) -> Dict[str, Any]:
    """Handle resume task - resumes audio playback from paused position."""
    
    try:
        if load_audio.nidaq_player is None:
            return create_error_response("No audio player initialized. Load audio first.")
        
        if not load_audio.nidaq_player._paused:
            if load_audio.nidaq_player._playing:
                return create_error_response("Audio is already playing.")
            else:
                return create_error_response("No audio currently paused. Use play to start playback.")
        
        load_audio.nidaq_player.resume()
        
        status = load_audio.nidaq_player.get_status()
        
        response_data = {
            "message": "Playback resumed",
            "status": status
        }
        
        return create_success_response(response_data)
        
    except Exception as e:
        return create_error_response(f"Resume task failed: {str(e)}")