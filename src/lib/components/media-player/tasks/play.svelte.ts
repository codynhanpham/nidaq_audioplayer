import { MediaPlayerData } from '@components/media-player';

export function playAudioHandler(websocket: WebSocket, message: string) {
    const data = JSON.parse(message);
    // console.log("Play audio handler received data:", data);
    if (data.status === "error") {
        console.error("Error occurred while playing audio:", data);
        MediaPlayerData.isPlaying = false;
        websocket.close();
        return;
    }

    if (data.id === "progress_update") {
        const playerInfo = data.data;
    
        MediaPlayerData.duration = playerInfo.duration;
        if (MediaPlayerData.audioInfo) {
            MediaPlayerData.audioInfo.duration = playerInfo.duration;
        }
        MediaPlayerData.isPlaying = playerInfo.playing;
        MediaPlayerData.playbackCompleted = playerInfo.audio_completed;

        if (!MediaPlayerData.pauseAutomaticWsProgressUpdate) {
            MediaPlayerData.progress = playerInfo.progress_percent;
        }
    }

    if (data.id === "playback_completed") {
        MediaPlayerData.isPlaying = false;
        MediaPlayerData.playbackCompleted = data.data.final_status.audio_completed;
        websocket.close();
    }

}