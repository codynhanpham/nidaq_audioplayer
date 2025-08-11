import datetime
import os
from typing import Dict, Any
from ..utils import create_success_response, create_error_response
from ..state import get_ws_start_time


async def handle_healthcheck(websocket, data: Any) -> Dict[str, Any]:
    """Handle healthcheck task - returns server status and uptime."""

    # Calculate uptime
    start_time = get_ws_start_time()
    if start_time:
        uptime_seconds = round((datetime.datetime.now() - start_time).total_seconds())
        uptime_str = str(datetime.timedelta(seconds=uptime_seconds))
    else:
        uptime_str = "unknown"

    try:
        import psutil
        
        # Get system information
        cpu_usage = psutil.cpu_percent(interval=0.1)
        memory_info = psutil.virtual_memory()
        
        health_data = {
            "server": "online",
            "pid": os.getpid(),
            "cpu_usage_percent": cpu_usage,
            "memory_usage_percent": memory_info.percent,
            "memory_available_gb": round(memory_info.available / (1024**3), 2),
            "uptime": uptime_str
        }
        
        return create_success_response(health_data)
        
    except ImportError:
        # Fallback if psutil is not available
        basic_health = {
            "server": "online",
            "pid": os.getpid(),
            "uptime": uptime_str,
            "message": "Basic health check - psutil not available for detailed metrics"
        }
        
        return create_success_response(basic_health)
        
    except Exception as e:
        return create_error_response(f"Health check failed: {str(e)}")
