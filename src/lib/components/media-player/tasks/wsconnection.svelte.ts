import { loadAudioHandler } from './load_audio.svelte'
import { playAudioHandler } from './play.svelte';
import { pauseAudioHandler } from './pause.svelte';
import { seekAudioHandler } from './seek.svelte';
import { volumeAudioHandler } from './volume.svelte';

export function wsSendOnce({ task, data }: { task: string; data?: any }) {
    const ws = new WebSocket("ws://localhost:21749");
    ws.onopen = () => {
        ws.send(JSON.stringify({ task, data }));
    };
    ws.onmessage = (event) => {
        switch (task) {
            case "load_audio":
                loadAudioHandler(ws, event.data);
                break;
            case "play":
                playAudioHandler(ws, event.data);
                break;
            case "pause":
                pauseAudioHandler(ws, event.data);
                break;
            case "seek":
                seekAudioHandler(ws, event.data);
                break;
            case "volume":
                volumeAudioHandler(ws, event.data);
                break;
            default:
                console.warn("Unknown task:", task);
                ws.close();
        }
    };
}