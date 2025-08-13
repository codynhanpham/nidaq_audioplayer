"""
WebSocket Server Utilities

Common utility functions used across task handlers.
"""

import uuid
import time
from typing import Dict, Any, Optional


from websockets.asyncio.server import broadcast
from . import ws


def create_response(
    status: str, 
    data: Any = None, 
    completed: bool = True, 
    last_msg_id: Optional[str] = None,
    task_name: Optional[str] = None
) -> Dict[str, Any]:
    """
    Create a standardized response format for WebSocket messages.
    
    Args:
        status: Response status ("success" or "error")
        data: Response data payload
        completed: Whether the operation is completed
        last_msg_id: ID of the last received message
    
    Returns:
        Standardized response dictionary
    """
    return {
        "id": str(uuid.uuid4()),
        "timestamp": int(time.time() * 1000),  # milliseconds
        "lastmsg": last_msg_id,
        "status": status,
        "data": data,
        "completed": completed,
        "task": task_name
    }


def create_error_response(error_message: str, last_msg_id: Optional[str] = None, task_name: Optional[str] = None) -> Dict[str, Any]:
    """
    Create a standardized error response.
    
    Args:
        error_message: Error message to include
        last_msg_id: ID of the last received message
    
    Returns:
        Standardized error response dictionary
    """
    return create_response(
        status="error",
        data={"error": error_message},
        completed=True,
        last_msg_id=last_msg_id,
        task_name=task_name
    )


def create_success_response(data: Any = None, last_msg_id: Optional[str] = None, task_name: Optional[str] = None) -> Dict[str, Any]:
    """
    Create a standardized success response.
    
    Args:
        data: Success data payload
        last_msg_id: ID of the last received message
    
    Returns:
        Standardized success response dictionary
    """
    return create_response(
        status="success",
        data=data,
        completed=True,
        last_msg_id=last_msg_id,
        task_name=task_name
    )


def validate_numeric_range(
    value: Any, 
    name: str, 
    min_val: float = None, 
    max_val: float = None,
    allow_none: bool = False
) -> tuple[bool, Optional[str]]:
    """
    Validate that a value is numeric and within specified range.
    
    Args:
        value: Value to validate
        name: Name of the field for error messages
        min_val: Minimum allowed value (inclusive)
        max_val: Maximum allowed value (inclusive)
        allow_none: Whether None values are allowed
    
    Returns:
        Tuple of (is_valid, error_message)
    """
    if value is None:
        if allow_none:
            return True, None
        return False, f"{name} cannot be None"
    
    if not isinstance(value, (int, float)):
        return False, f"{name} must be a number"
    
    if min_val is not None and value < min_val:
        return False, f"{name} must be >= {min_val}"
    
    if max_val is not None and value > max_val:
        return False, f"{name} must be <= {max_val}"
    
    return True, None


def validate_required_fields(data: Dict[str, Any], required_fields: list[str]) -> tuple[bool, Optional[str]]:
    """
    Validate that required fields are present in data.
    
    Args:
        data: Data dictionary to validate
        required_fields: List of required field names
    
    Returns:
        Tuple of (is_valid, error_message)
    """
    if not data:
        return False, f"Missing required fields: {', '.join(required_fields)}"
    
    missing_fields = [field for field in required_fields if field not in data]
    
    if missing_fields:
        return False, f"Missing required fields: {', '.join(missing_fields)}"
    
    return True, None


def safe_get_nested(data: Dict[str, Any], keys: list[str], default: Any = None) -> Any:
    """
    Safely get nested dictionary values.
    
    Args:
        data: Dictionary to search in
        keys: List of keys for nested access
        default: Default value if key path doesn't exist
    
    Returns:
        Value at the key path or default
    """
    current = data
    try:
        for key in keys:
            current = current[key]
        return current
    except (KeyError, TypeError):
        return default


async def broadcast_message(message: Dict[str, Any]):
    """Broadcast a message to all connected WebSocket clients."""
    await ws.broadcast(ws.CONNECTIONS, message)