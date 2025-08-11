from typing import Dict, Any
from .load_audio import nidaq_player
from ..utils import create_success_response, create_error_response


async def handle_status(websocket, data: Any) -> Dict[str, Any]:
    """Handle status task - returns current player status."""
    global nidaq_player
    
    try:
        if nidaq_player is None:
            response_data = {
                "message": "No audio player initialized",
                "status": None
            }
        else:
            # Get detailed status from player
            status = nidaq_player.get_status()
            response_data = {
                "message": "Current player status",
                "status": status
            }
            
        
        return create_success_response(response_data)
        
    except Exception as e:
        return create_error_response(f"Status task failed: {str(e)}")