import os
import asyncio
from websockets.asyncio.server import serve


# async def echo(websocket):
#     async for message in websocket:
#         await websocket.send(message)

async def handle_message(websocket):
    """Handle incoming WebSocket messages and route to appropriate functions."""
    async for message in websocket:
        message = message.strip()
        
        if message == "#!pid":
            pid = str(os.getpid())
            await websocket.send(pid)
        elif message == "#!terminate":
            await websocket.send("Server is shutting down.")
            await websocket.close()
            exit(0)
        else:
            # Echo unknown messages or handle other commands
            await websocket.send(f"Unknown command: {message}")



async def main():
    async with serve(handle_message, "localhost", 21749) as server:
        print("WebSocket server started on ws://localhost:21749")
        await server.serve_forever()

def start_websocket_server():
    """Start the WebSocket server."""
    asyncio.run(main())