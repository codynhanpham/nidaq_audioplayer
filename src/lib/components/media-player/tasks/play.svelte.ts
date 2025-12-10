import { MediaPlayerData } from '@components/media-player';
export const updateStatus = {
    lastUpdated: 0,
    interval: 330, // milliseconds
}

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
        // Use request animation frame to update UI smoothly
        const updateProgress = () => {
            const playerInfo = data.data;
            if (!playerInfo) {
                return;
            }

            if (Date.now() - updateStatus.lastUpdated < updateStatus.interval) {
                return;
            }
        
            MediaPlayerData.duration = playerInfo.duration;
            if (MediaPlayerData.audioInfo) {
                MediaPlayerData.audioInfo.duration = playerInfo.duration;
            }
            MediaPlayerData.isPlaying = playerInfo.playing;
            MediaPlayerData.playbackCompleted = playerInfo.audio_completed;

            if (!MediaPlayerData.pauseAutomaticWsProgressUpdate) {
                MediaPlayerData.progress = playerInfo.progress_percent;
            }
            updateStatus.lastUpdated = Date.now();
        }
        requestAnimationFrame(updateProgress);
        return;
    }

    if (data.id === "playback_completed") {
        MediaPlayerData.isPlaying = false;
        MediaPlayerData.playbackCompleted = data.data.final_status.audio_completed;
        MediaPlayerData.progress = 100;
        MediaPlayerData.duration = data.data.final_status.duration;
        websocket.close();
    }
}