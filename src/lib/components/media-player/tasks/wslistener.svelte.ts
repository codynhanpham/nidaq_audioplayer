import { MediaPlayerData } from '../playerData.svelte';

/**  A dedicated ws message listener that handles syncing MediaPlayerData state  */
let wsEventListener: WebSocket | undefined = $state(undefined);

export function wsEventListenerExists(): boolean {
    return wsEventListener !== undefined;
}

export function initWsEventListener() {
    wsEventListener = new WebSocket("ws://localhost:21749");

    wsEventListener.onmessage = (event) => {
        console.log(event);
    };
    console.log("WebSocket Player Event Listeners initialized");
}

export function closeWsEventListener() {
    if (wsEventListener) {
        wsEventListener.close();
        wsEventListener = undefined;
    }
}