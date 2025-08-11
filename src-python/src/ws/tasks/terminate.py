import sys
import json
from typing import Dict, Any
from ..utils import create_success_response, create_error_response


async def handle_terminate(websocket, data: Any) -> Dict[str, Any]:
    """Handle terminate task - gracefully shuts down the server."""
    try:
        response = create_success_response({"message": "Server is shutting down gracefully"})
        
        await websocket.send(json.dumps(response))
        await websocket.close()
        
        sys.exit(0)
        
    except Exception as e:
        return create_error_response(f"Failed to terminate server: {str(e)}")
