import os
from typing import Dict, Any
from ..utils import create_success_response, create_error_response


async def handle_pid(websocket, data: Any) -> Dict[str, Any]:
    """Handle pid task - returns the current process ID."""
    try:
        pid = os.getpid()
        
        pid_data = {
            "pid": pid,
        }
        
        return create_success_response(pid_data)
        
    except Exception as e:
        return create_error_response(f"Failed to get PID: {str(e)}")
