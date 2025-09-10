import { MediaPlayerData } from '@components/media-player';

export function loadAudioHandler(websocket: WebSocket, message: string) {
    const data = JSON.parse(message);

    // console.log(data);
    
    if (data.status === "error") {
        console.error("Error occurred while loading audio:", data);
        return;
    }

    const playerInfo = data.data.player_info;

    MediaPlayerData.duration = playerInfo.duration;
    if (MediaPlayerData.audioInfo) {
        MediaPlayerData.audioInfo.duration = playerInfo.duration;
    }
    MediaPlayerData.isPlaying = playerInfo.playing;
    MediaPlayerData.progress = playerInfo.sample_generated / playerInfo.total_audio_samples * 100;
    MediaPlayerData.volume = playerInfo.volume;
    MediaPlayerData.flipLRStereo = playerInfo.flip_lr_stereo || false;

    websocket.close();
}