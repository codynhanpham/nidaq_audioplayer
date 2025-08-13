from typing import Dict, Any
from . import load_audio
from ..utils import create_success_response, create_error_response, validate_numeric_range, validate_required_fields


async def handle_volume(websocket, data: Any) -> Dict[str, Any]:
    """Handle volume task - sets voltage scale (volume) from 0-100% to 0-1V."""
    
    try:
        if load_audio.nidaq_player is None:
            return create_error_response("No audio player initialized. Load audio first.")
        
        # Validate required fields
        is_valid, error_msg = validate_required_fields(data, ["volume"])
        if not is_valid:
            return create_error_response(error_msg)
        
        volume = data["volume"]
        
        is_valid, error_msg = validate_numeric_range(volume, "volume", min_val=0, max_val=100)
        if not is_valid:
            return create_error_response(error_msg)
        
        voltage_scale = volume / 100.0
        
        # Update the player's voltage scale using the proper method
        old_voltage_scale = load_audio.nidaq_player.voltage_scale
        load_audio.nidaq_player.update_voltage_scale(voltage_scale)
        
        response_data = {
            "message": f"Volume set to {volume}% (voltage scale: {voltage_scale:.3f}V)",
            "volume_percent": volume,
            "voltage_scale": voltage_scale,
            "previous_voltage_scale": old_voltage_scale,
            "status": load_audio.nidaq_player.get_status()
        }
        
        return create_success_response(response_data)
        
    except Exception as e:
        return create_error_response(f"Volume task failed: {str(e)}")