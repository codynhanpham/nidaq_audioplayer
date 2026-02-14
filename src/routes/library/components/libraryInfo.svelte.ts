import { load, type Store } from '@tauri-apps/plugin-store';
import { invoke } from "@tauri-apps/api/core";

export const libraryStore = load('library.json', {
    autoSave: false,
    defaults: {}
}).then((data) => {
    return data as Store;
});

let lastStoreUpdated = $state(Date.now());
export function getLastStoreUpdated() {
    return lastStoreUpdated;
}

export type LibraryDirInfo = {
    dir: string;
    fileCount: number;
};

export type Library = {
    audioFiles: string[];
    libraryStats: LibraryDirInfo[];
};

export async function updateLibraryStore(store: Store, data: Library) {
    await store.set('library', data);
    lastStoreUpdated = Date.now();
    await store.save();
}

export async function listLibraryDirs(store: Store): Promise<string[]> {
    const library = await store.get('library') as Library | undefined;
    if (library) {
        return library.libraryStats.map((info) => info.dir);
    }
    return [];
}

export async function setScanRecursiveLevel(store: Store, level: number): Promise<void> {
    await store.set('scanRecursiveLevel', level);
    await store.save();
}
export async function getScanRecursiveLevel(store: Store): Promise<number> {
    let value = await store.get('scanRecursiveLevel');
    if (typeof value === 'number') {
        return value;
    }
    else {
        await setScanRecursiveLevel(store, 4);
        return 4;
    }
}

export async function rescanLibrary(store: Store): Promise<Library | undefined> {
    const dirs = await listLibraryDirs(store);
    const recursiveLevel = await getScanRecursiveLevel(store);
    const scanResult = await invoke("flex_search_audio_files", {
        paths: dirs,
        recursiveLevel: recursiveLevel,
    }) as any;
    if (scanResult) {
        const library: Library = {
            audioFiles: scanResult.paths,
            libraryStats: scanResult.stats
        };
        await updateLibraryStore(store, library);
    }
    return scanResult ? {
        audioFiles: scanResult.paths,
        libraryStats: scanResult.stats
    } : undefined;
}


export async function setLastLibbinHash(store: Store, hash: string): Promise<void> {
    await store.set('lastLibbinHash', hash);
    await store.save();
}
export async function getLastLibbinHash(store: Store): Promise<string | undefined> {
    return await store.get('lastLibbinHash');
}