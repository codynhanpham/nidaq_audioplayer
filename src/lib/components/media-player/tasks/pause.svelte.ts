import { MediaPlayerData } from '@components/media-player';

export function pauseAudioHandler(websocket: WebSocket, message: string) {
    const data = JSON.parse(message);

    if (data.status === "error") {
        console.error("Error occurred while pausing audio:", data);
        return;
    }

    const playerInfo = data.status;

    MediaPlayerData.isPlaying = playerInfo.playing;

    websocket.close();
}