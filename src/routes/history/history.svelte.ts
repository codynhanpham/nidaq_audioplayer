import { type AudioInfo } from "@components/media-player";
import { load, type Store } from '@tauri-apps/plugin-store';

export type HistoryData = {
    audioInfo: AudioInfo[] | null; // List of recently played audio files, most recent at the end of the array, or null if no history
}

export const HistoryAudioInfo: { audioInfo: AudioInfo[] | null } = {
    audioInfo: null,
};

export const historyStore = load('history.json', {
    autoSave: false,
    defaults: {}
}).then((data) => {
    return data as Store;
});

function saveHistoryToStore(store: Store) {
    store.set('history', HistoryAudioInfo.audioInfo);
    store.save();
}
export async function loadHistoryFromStore(store: Store) {
    const history = await store.get('history') as AudioInfo[] | null;
    HistoryAudioInfo.audioInfo = history;
}


export function addToHistory(audioInfo: AudioInfo) {
    if (!HistoryAudioInfo.audioInfo) {
        HistoryAudioInfo.audioInfo = [];
    }

    // Push without duplicates, most recent at the end of the array
    const existingIndex = HistoryAudioInfo.audioInfo.findIndex(info => info.path === audioInfo.path);
    if (existingIndex !== -1) {
        HistoryAudioInfo.audioInfo.splice(existingIndex, 1); // Remove existing entry
    }
    HistoryAudioInfo.audioInfo.push(audioInfo);

    // Keep only the most recent 50 entries (audioInfo also stores the thumbnail which can take up a lot of diskspace)
    if (HistoryAudioInfo.audioInfo.length > 50) {
        HistoryAudioInfo.audioInfo = HistoryAudioInfo.audioInfo.slice(-50);
    }

    // Save updated history to store
    historyStore.then(store => {
        saveHistoryToStore(store);
    });
}

/**
 * Cleanup history by removing entries that were removed from Library Location or no longer exist on disk
 * Generally, this should be called when the master AudioInfo list is updated, which happens when the user opens the app or changes the Library Location
 */
export function cleanupHistory(currentAudioInfoList: AudioInfo[] | null) {
    if (!HistoryAudioInfo.audioInfo) {
        return;
    }

    if (!currentAudioInfoList) {
        // If there is no current audio info list, clear the history
        HistoryAudioInfo.audioInfo = null;
        historyStore.then(store => {
            saveHistoryToStore(store);
        });
        return;
    }

    // Remove any history entries that are not in the current audio info list
    HistoryAudioInfo.audioInfo = HistoryAudioInfo.audioInfo.filter(info => {
        return currentAudioInfoList.some(current => current.path === info.path);
    });
    // Save updated history to store
    historyStore.then(store => {
        saveHistoryToStore(store);
    });
}

export function clearHistory() {
    HistoryAudioInfo.audioInfo = null;
    historyStore.then(store => {
        saveHistoryToStore(store);
    });
}