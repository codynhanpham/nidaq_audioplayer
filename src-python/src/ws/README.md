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
- `device_name`: NI-DAQ device name
- `ao_channels`: List of analog output channels

**Data Optional:**
- `ai_channels`: List of analog input channels
- `do_channels`: List of digital output channels
- `volume`: Volume level 0-100 (default: 20)
- `samples_per_frame`: Samples per frame (default: 8192)
- `flip_lr_stereo`: Whether to flip left/right stereo channels (default: false)

```json
{
  "task": "load_audio",
  "data": {
    "file_path": "/path/to/audio.wav",
    "device_name": "Dev1",
    "ao_channels": ["/ao0", "/ao1"],
    "flip_lr_stereo": false
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

### 12. Flip Left/Right Stereo Channels
**Task:** `flip_lr_stereo`
**Data Optional:**
- `flip_lr_stereo`: Boolean to enable/disable L/R channel flipping (if omitted, returns current setting)

**Description:** Toggles left/right stereo channel flipping. Only applies to audio files with exactly 2 channels.

```json
{
  "task": "flip_lr_stereo",
  "data": {
    "flip_lr_stereo": true
  }
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
play_result = await send_command("play")
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
sendCommand('play');
```

Make sure the WebSocket server is running first.
