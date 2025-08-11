from typing import Dict, Any
from .load_audio import nidaq_player
from ..utils import create_success_response, create_error_response, validate_numeric_range, validate_required_fields


async def handle_seek(websocket, data: Any) -> Dict[str, Any]:
    """Handle seek task - seeks to a specific position in the audio file."""
    global nidaq_player
    
    try:
        if nidaq_player is None:
            return create_error_response("No audio player initialized. Load audio first.")
        
        if not nidaq_player._audio_loaded:
            return create_error_response("No audio file loaded. Load audio first.")
        
        # Validate required fields - accept both 'position' and 'time' for flexibility
        position_field = None
        if "position" in data:
            position_field = "position"
        elif "time" in data:
            position_field = "time"
        else:
            return create_error_response("Missing required field: 'position' or 'time' (time in seconds)")
        
        position = data[position_field]
        
        # Validate position range
        max_duration = nidaq_player._audio_duration if hasattr(nidaq_player, '_audio_duration') else 0
        is_valid, error_msg = validate_numeric_range(position, position_field, min_val=0, max_val=max_duration)
        if not is_valid:
            return create_error_response(error_msg)
        
        # Perform seek
        old_status = nidaq_player.get_status()
        old_position = old_status.get('current_time', 0)
        
        nidaq_player.seek(position)
        
        new_status = nidaq_player.get_status()
        
        response_data = {
            "message": f"Seeked from {old_position:.2f}s to {position:.2f}s",
            "old_position": old_position,
            "new_position": position,
            "duration": max_duration,
            "status": new_status
        }
        
        return create_success_response(response_data)
        
    except Exception as e:
        return create_error_response(f"Seek task failed: {str(e)}")