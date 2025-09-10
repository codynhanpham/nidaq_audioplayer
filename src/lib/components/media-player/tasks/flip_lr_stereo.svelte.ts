import { MediaPlayerData } from '@components/media-player';

export function flipLRStereoHandler(websocket: WebSocket, message: string) {
    const data = JSON.parse(message);

    if (data.status === "error") {
        console.error("Error occurred while toggling flip L/R stereo:", data);
        return;
    }

    // Update the MediaPlayerData with the new flip L/R stereo setting
    if (data.data && typeof data.data.flip_lr_stereo === 'boolean') {
        MediaPlayerData.flipLRStereo = data.data.flip_lr_stereo;
    }

    websocket.close();
}
