import { MediaPlayerData } from '@components/media-player';

export function volumeAudioHandler(websocket: WebSocket, message: string) {
    const data = JSON.parse(message);

    if (data.status === "error") {
        console.error("Error occurred while changing volume:", data);
        return;
    }

    // DO NOT UPDATE FROM MESSAGE (FOR NOW) AS MUTE ALSO PIGGY BACK THIS FUNCTION
    // SHOULDN'T BE AN ISSUE, IN ANY CASE
    // MediaPlayerData.volume = data.data.volume_percent;

    websocket.close();
}