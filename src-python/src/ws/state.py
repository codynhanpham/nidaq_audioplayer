"""WebSocket server state management."""
import datetime
from typing import Optional

# Global server start time
_ws_start_time: Optional[datetime.datetime] = None


def set_ws_start_time(start_time: datetime.datetime) -> None:
    """Set the WebSocket server start time."""
    global _ws_start_time
    _ws_start_time = start_time


def get_ws_start_time() -> Optional[datetime.datetime]:
    """Get the WebSocket server start time."""
    return _ws_start_time
