import json
import uuid
import time
from typing import Dict, Any, Optional
from .tasks.healthcheck import handle_healthcheck
from .tasks.pid import handle_pid
from .tasks.terminate import handle_terminate
from .tasks.load_audio import handle_load_audio
from .tasks.play import handle_play
from .tasks.pause import handle_pause, handle_resume
from .tasks.status import handle_status
from .tasks.set_voltage_scale import handle_volume
from .tasks.seek import handle_seek
from .tasks.flip_lr_stereo import handle_flip_lr_stereo


class MessageHandler:
    """Handles WebSocket message routing and response formatting."""
    
    def __init__(self):
        self.task_handlers = {
            "healthcheck": handle_healthcheck,
            "pid": handle_pid,
            "terminate": handle_terminate,
            "load_audio": handle_load_audio,
            "play": handle_play,
            "pause": handle_pause,
            "resume": handle_resume,
            "status": handle_status,
            "volume": handle_volume,
            "seek": handle_seek,
            "flip_lr_stereo": handle_flip_lr_stereo,
        }
        self.last_message_id: Optional[str] = None
    
    def create_response(
        self,
        status: str,
        data: Any = None,
        completed: bool = True,
    ) -> Dict[str, Any]:
        """Create a standardized response format."""
        response = {
            "id": str(uuid.uuid4()),
            "timestamp": int(time.time() * 1000),  # milliseconds
            "lastmsg": self.last_message_id,
            "status": status,
            "data": data,
            "completed": completed
        }
        return response
    
    async def handle_message(self, websocket, message: str) -> Dict[str, Any]:
        """Parse and handle incoming WebSocket messages."""
        try:
            # Parse JSON message
            parsed_message = json.loads(message)
            
            # Update last message ID if provided
            if "id" in parsed_message:
                self.last_message_id = parsed_message["id"]
            
            # Extract task and data
            task = parsed_message.get("task")
            data = parsed_message.get("data")
            
            if not task:
                return self.create_response(
                    "error",
                    {"error": "Missing 'task' field in message"},
                    True
                )
            
            # Route to appropriate task handler
            if task in self.task_handlers:
                try:
                    result = await self.task_handlers[task](websocket, data)
                    return result
                except Exception as e:
                    return self.create_response(
                        "error",
                        {"error": f"Task '{task}' failed: {str(e)}"},
                        True
                    )
            else:
                return self.create_response(
                    "error",
                    {"error": f"Unknown task: {task}"},
                    True
                )
                
        except json.JSONDecodeError as e:
            return self.create_response(
                "error",
                {"error": f"Invalid JSON: {str(e)}"},
                True
            )
        except Exception as e:
            return self.create_response(
                "error",
                {"error": f"Unexpected error: {str(e)}"},
                True
            )
