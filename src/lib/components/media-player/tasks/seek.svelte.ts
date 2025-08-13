import { MediaPlayerData } from '@components/media-player';

export function seekAudioHandler(websocket: WebSocket, message: string) {
    const data = JSON.parse(message);

    if (data.status === "error") {
        console.error("Error occurred while seeking audio:", data);
        return;
    }

    const playerInfo = data.status;
    MediaPlayerData.isPlaying = playerInfo.playing;
    MediaPlayerData.progress = playerInfo.sample_generated / playerInfo.total_audio_samples * 100;

    websocket.close();
}