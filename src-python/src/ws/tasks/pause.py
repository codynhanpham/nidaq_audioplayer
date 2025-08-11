from typing import Dict, Any
from .load_audio import nidaq_player
from ..utils import create_success_response, create_error_response


async def handle_pause(websocket, data: Any) -> Dict[str, Any]:
    """Handle pause task - pauses audio playback."""
    global nidaq_player
    
    try:
        if nidaq_player is None:
            return create_error_response("No audio player initialized. Load audio first.")
        
        if not nidaq_player._playing:
            return create_error_response("No audio currently playing.")
        
        nidaq_player.pause()
        
        status = nidaq_player.get_status()
        
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
    global nidaq_player
    
    try:
        if nidaq_player is None:
            return create_error_response("No audio player initialized. Load audio first.")
        
        if not nidaq_player._paused:
            if nidaq_player._playing:
                return create_error_response("Audio is already playing.")
            else:
                return create_error_response("No audio currently paused. Use play to start playback.")
        
        nidaq_player.resume()
        
        status = nidaq_player.get_status()
        
        response_data = {
            "message": "Playback resumed",
            "status": status
        }
        
        return create_success_response(response_data)
        
    except Exception as e:
        return create_error_response(f"Resume task failed: {str(e)}")