import os, datetime
import json
import asyncio
from websockets.asyncio.server import serve
from .message_handler import MessageHandler
from .state import set_ws_start_time

async def handle_message(websocket):
    """Handle incoming WebSocket messages and route to appropriate functions."""
    handler = MessageHandler()
    
    async for message in websocket:
        message = message.strip()
        
        # Handle JSON messages
        try:
            response = await handler.handle_message(websocket, message)
            await websocket.send(json.dumps(response))
        except Exception as e:
            error_response = {
                "id": "error",
                "timestamp": int(asyncio.get_event_loop().time() * 1000),
                "lastmsg": None,
                "status": "error",
                "data": {"error": f"Message handling failed: {str(e)}"},
                "completed": True
            }
            await websocket.send(json.dumps(error_response))


async def main():
    async with serve(handle_message, "localhost", 21749) as server:
        set_ws_start_time(datetime.datetime.now())
        print("WebSocket server started on ws://localhost:21749")
        await server.serve_forever()

def start_websocket_server():
    """Start the WebSocket server."""
    asyncio.run(main())