# WebSocket Audio Server

A JSON-based WebSocket server for controlling audio playback with standardized request/response formats.

## Message Format

### Request Format
```json
{
  "id": "optional-message-id",
  "task": "task_name",
  "data": {
    // task-specific data
  }
}
```

### Response Format
```json
{
  "id": "response-uuid",
  "timestamp": 1625097600000,
  "lastmsg": "last-message-id",
  "status": "success" | "error",
  "data": {
    // response data
  },
  "completed": true | false
}
```

## Available Tasks

### 1. Health Check
**Task:** `healthcheck`
**Data:** None required
**Description:** Returns server health status and system information

```json
{
  "task": "healthcheck"
}
```

### 2. Get Process ID
**Task:** `pid`
**Data:** None required
**Description:** Returns the server process ID

```json
{
  "task": "pid"
}
```

### 3. Server Status
**Task:** `status`
**Data:** None required
**Description:** Returns comprehensive server and audio status

```json
{
  "task": "status"
}
```

### 4. Terminate Server
**Task:** `terminate`
**Data:** None required
**Description:** Gracefully shuts down the server

```json
{
  "task": "terminate"
}
```

### 5. Load Audio File
**Task:** `load_audio`
**Data Required:**
- `file_path`: Path to audio file

```json
{
  "task": "load_audio",
  "data": {
    "file_path": "/path/to/audio.wav"
  }
}
```

### 6. Play Audio
**Task:** `play`
**Data Optional:**
- `start_position`: Start position in seconds (default: 0.0)
- `volume`: Volume level 0.0-1.0 (default: 1.0)
- `loop`: Whether to loop playback (default: false)

```json
{
  "task": "play",
  "data": {
    "start_position": 0.0,
    "volume": 0.8,
    "loop": false
  }
}
```

### 7. Pause/Stop Audio
**Task:** `pause`
**Data Optional:**
- `stop`: If true, stops completely; if false, pauses (default: false)

```json
{
  "task": "pause",
  "data": {
    "stop": false
  }
}
```

### 8. Resume Audio
**Task:** `resume`
**Data:** None required
**Description:** Resumes paused audio playback

```json
{
  "task": "resume"
}
```

### 9. Volume Control
**Task:** `volume`
**Data Optional:**
- `volume`: New volume level 0.0-1.0 (if omitted, returns current volume)

```json
{
  "task": "volume",
  "data": {
    "volume": 0.5
  }
}
```

### 10. Seek to Position
**Task:** `seek`
**Data Required:**
- `position`: Position in seconds to seek to

```json
{
  "task": "seek",
  "data": {
    "position": 45.5
  }
}
```

### 11. Get Current Position
**Task:** `get_position`
**Data:** None required
**Description:** Returns current playback position and audio info

```json
{
  "task": "get_position"
}
```

## Usage Examples

### Python Client
```python
import asyncio
import json
import websockets

async def send_command(task, data=None):
    uri = "ws://localhost:21749"
    async with websockets.connect(uri) as websocket:
        message = {"task": task}
        if data:
            message["data"] = data
        
        await websocket.send(json.dumps(message))
        response = await websocket.recv()
        return json.loads(response)

# Examples
health = await send_command("healthcheck")
status = await send_command("status")
load_result = await send_command("load_audio", {"file_path": "music.wav"})
play_result = await send_command("play", {"volume": 0.8})
```

### JavaScript Client
```javascript
const ws = new WebSocket('ws://localhost:21749');

function sendCommand(task, data = null) {
    const message = { task };
    if (data) message.data = data;
    
    ws.send(JSON.stringify(message));
}

ws.onmessage = (event) => {
    const response = JSON.parse(event.data);
    console.log('Response:', response);
};

// Examples
sendCommand('healthcheck');
sendCommand('load_audio', { file_path: 'music.wav' });
sendCommand('play', { volume: 0.8, loop: true });
```

## File Structure

```
src/ws/
├── __init__.py
├── ws.py                 # Main WebSocket server
├── message_handler.py    # JSON message routing
├── utils.py             # Shared utility functions
├── config.py            # Configuration settings
├── client_example.py     # Test client example
├── README.md            # This file
└── tasks/               # Individual task handlers
    ├── __init__.py
    ├── healthcheck.py   # Health check task
    ├── pid.py          # Process ID task
    ├── terminate.py    # Server termination task
    ├── load_audio.py   # Audio loading task
    ├── play.py         # Audio playback task
    ├── pause.py        # Pause/resume tasks
    ├── seek.py         # Seeking and position tasks
    └── status.py       # Status and volume tasks
```

## Code Architecture

The WebSocket server uses a modular architecture with shared utilities to avoid code duplication:

### Core Components

- **`utils.py`** - Shared utility functions including:
  - `create_response()` - Standardized response formatting
  - `create_success_response()` - Success response helper
  - `create_error_response()` - Error response helper
  - `validate_numeric_range()` - Numeric validation
  - `validate_required_fields()` - Field validation
  - `safe_get_nested()` - Safe dictionary access

- **`message_handler.py`** - Routes incoming JSON messages to appropriate task handlers

- **`tasks/`** - Individual task modules that import utilities for consistent responses

## Starting the Server

```python
from src.ws.ws import start_websocket_server
start_websocket_server()
```

The server will start on `ws://localhost:21749` and support both legacy text commands and new JSON messages.

## Legacy Support

The server still supports legacy text commands for backward compatibility:
- `#!pid` - Returns process ID (plain text)
- `#!terminate` - Terminates server

## Integration Notes

This implementation provides the framework for audio control. To integrate with actual audio playback:

1. Install audio libraries (e.g., `pygame`, `pydub`, `soundfile`)
2. Replace placeholder audio operations in task files
3. Implement actual audio loading, playback, and control logic
4. Add error handling for audio-specific exceptions

## Testing

Run the test client:
```bash
python src/ws/client_example.py
```

Make sure the WebSocket server is running first.
