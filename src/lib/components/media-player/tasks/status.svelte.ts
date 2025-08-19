import { MediaPlayerData } from '@components/media-player';

export function playerStatusHandler(websocket: WebSocket, message: string) {
    const data = JSON.parse(message);

    if (data.status === "error") {
        console.error("Error occurred while changing volume:", data);
        return;
    }
    
    console.log(data)

    websocket.close();
}