from typing import Dict, Any
from . import load_audio
from ..utils import create_success_response, create_error_response, validate_required_fields


async def handle_flip_lr_stereo(websocket, data: Any) -> Dict[str, Any]:
    """Handle flip_lr_stereo task - toggles left/right stereo channel flipping."""
    
    try:
        if load_audio.nidaq_player is None:
            return create_error_response("No audio player initialized. Load audio first.")
        
        # Check if data contains the flip_lr_stereo field, if not, just return current state
        if data is None or "flip_lr_stereo" not in data:
            current_setting = load_audio.nidaq_player.get_flip_lr_stereo()
            response_data = {
                "message": f"Current flip L/R stereo setting: {current_setting}",
                "flip_lr_stereo": current_setting,
                "status": load_audio.nidaq_player.get_status()
            }
            return create_success_response(response_data)
        
        # Validate that flip_lr_stereo is a boolean
        flip_lr_stereo = data["flip_lr_stereo"]
        if not isinstance(flip_lr_stereo, bool):
            return create_error_response("flip_lr_stereo must be a boolean value (true or false)")
        
        # Update the player's flip L/R stereo setting
        old_setting = load_audio.nidaq_player.get_flip_lr_stereo()
        load_audio.nidaq_player.set_flip_lr_stereo(flip_lr_stereo)
        
        response_data = {
            "message": f"Flip L/R stereo setting updated from {old_setting} to {flip_lr_stereo}",
            "flip_lr_stereo": flip_lr_stereo,
            "previous_setting": old_setting,
            "status": load_audio.nidaq_player.get_status()
        }
        
        return create_success_response(response_data)
        
    except Exception as e:
        return create_error_response(f"Flip L/R stereo task failed: {str(e)}")
