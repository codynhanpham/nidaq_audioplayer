import asyncio
from websockets.asyncio.server import serve


async def echo(websocket):
    async for message in websocket:
        await websocket.send(message)


async def main():
    async with serve(echo, "localhost", 21749) as server:
        print("WebSocket server started on ws://localhost:21749")
        await server.serve_forever()

def start_websocket_server():
    """Start the WebSocket server."""
    asyncio.run(main())