import { MediaPlayerData } from '@components/media-player';

export function seekAudioHandler(websocket: WebSocket, message: string) {
    const data = JSON.parse(message);

    if (data.status === "error") {
        console.error("Error occurred while seeking audio:", data);
        return;
    }

    const playerInfo = data.data.status;
    MediaPlayerData.isPlaying = playerInfo.playing;
    MediaPlayerData.progress = playerInfo.current_time / playerInfo.duration * 100;

    websocket.close();
}