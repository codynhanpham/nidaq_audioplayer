<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";

	import { getCurrentWindow } from '@tauri-apps/api/window';
    import { sep } from '@tauri-apps/api/path';

    import { Button } from '$lib/components/ui/button/index.js'
    import { Skeleton } from "$lib/components/ui/skeleton/index.js";


    import { getCurrentWebview } from "@tauri-apps/api/webview";
    import type { UnlistenFn } from "@tauri-apps/api/event";
    import { onMount } from "svelte";


    import {
        FileMusic
    } from "@lucide/svelte/icons";


    import { libraryStore, type Library, rescanLibrary, getLastStoreUpdated } from "./components/index.js";
    (async () => {
        rescanLibrary(await libraryStore);
    })();
    import LibrarySelector from './components/library-selector.svelte';
    import { type AudioInfo, MediaPlayerData } from "$lib/components/media-player/playerData.svelte.js";
	import { wsSendOnce } from '@components/media-player/tasks/wsconnection.svelte';
    import { StatusBarData } from "@components/app-statusbar";


    let audioFilesList: Promise<string[]> = $derived.by(async () => {
        getLastStoreUpdated(); // Force update from store when it's updated
        const store = await libraryStore;
		const libraryData = await store.get<Library>('library');
		if (!libraryData) return [];
		return libraryData.audioFiles;
	});

    async function getMediaInfo(file: string): Promise<AudioInfo | undefined> {
        try {
            const metadata = await invoke("get_media_metadata", { path: file }) as AudioInfo | undefined;
            return metadata;
        }
        catch {
            console.error("Failed to retrieve media info for:", file);
            return undefined;
        }
    }
    let audioFiles: Promise<(AudioInfo | undefined)[] | null> = $derived.by(async () => {
        const files = await audioFilesList;
        if (!files) return null;
        const content = await Promise.all(files.map(getMediaInfo));
        content.sort((a, b) => {
            const nameA = a?.name.toLowerCase() || "";
            const nameB = b?.name.toLowerCase() || "";
            return nameA.localeCompare(nameB);
        });
        return content;
    });

    function handleAudioTrackSelect(data: AudioInfo | undefined) {
        if (!data) {
            return;
        }

        if (!StatusBarData.niDaqSelectedDevice?.name) {
            console.warn("No NI-DAQ device selected");
            // TODO: Handle show UI notice here!
            return;
        }

        MediaPlayerData.audioInfo = data;
        MediaPlayerData.duration = data.duration;
        wsSendOnce({
            task: "load_audio",
            data: {
                device_name: StatusBarData.niDaqSelectedDevice?.name,
                file_path: data.path,
                ao_channels: ['/ao0', '/ao1'],
                ai_channels: ['/ai0', '/ai1'],
                do_channels: ['/port0/line0', '/port0/line1'],
                volume: MediaPlayerData.volume,
                samples_per_frame: 4096,
            }
        });
        wsSendOnce({
            task: "play"
        });
    }
    function handleArtistSelect(data: AudioInfo | undefined) {
        if (!data) {
            return;
        }
        
    }

</script>

<main class="w-full h-full min-h-fit space-y-2 p-2">
    <LibrarySelector />

    {#await audioFiles}
        <ul class="grid gap-x-4 gap-y-4 grid-cols-[repeat(auto-fill,minmax(9.5rem,1fr))] md:grid-cols-[repeat(auto-fill,minmax(10rem,1fr))] my-3 mx-2">
            {#each Array.from({ length: 12 }) as _, i}
                <li class="w-full h-full flex flex-col items-center justify-start">
                    <Skeleton class="p-0 !w-full h-full !aspect-square rounded-sm overflow-hidden" title={`Loading...`} />
                    <div class="w-full h-fit text-center flex flex-col items-center justify-start">
                        <Skeleton class="px-1 pt-1.5 pb-0.5 inline mt-2 w-3/4 !h-4 max-w-full text-sm text-wrap line-clamp-1 text-ellipsis" />
                        <Skeleton class="text-xs text-muted-foreground mt-1 w-1/2 !h-3 hover:text-foreground/80" />
                    </div>
                </li>
            {/each}
        </ul>
    {:then audioFiles}
        {#if audioFiles && audioFiles.length > 0}
            <ul class="grid gap-x-4 gap-y-4 grid-cols-[repeat(auto-fill,minmax(9.5rem,1fr))] md:grid-cols-[repeat(auto-fill,minmax(10rem,1fr))] my-3 mx-2">
                {#each audioFiles as file}
                    {#if file}
                        <!-- <li>{file.name || file.path.split("\\").pop()}</li> -->
                        <li class="w-full h-full flex flex-col items-center justify-start">
                            {#if file.thumbnail}
                                <Button variant="ghost" class="p-0 w-full h-fit rounded-sm overflow-hidden" title={file.name || file.path.split(sep()).pop()} onclick={() => handleAudioTrackSelect(file)}>
                                    <img src={file.thumbnail} alt={`${file.name || file.path.split(sep()).pop()} cover image`} class="w-full h-fit aspect-square object-cover" />
                                </Button>
                            {:else}
                                <Button variant="ghost" class="w-full h-fit aspect-square flex items-center justify-center bg-muted rounded-sm" title={file.name || file.path.split(sep()).pop()} onclick={() => handleAudioTrackSelect(file)}>
                                    <FileMusic class="w-1/3 h-fit aspect-square" />
                                </Button>
                            {/if}
                            <div class="w-full h-fit text-center flex flex-col items-center justify-start">
                                <Button variant="link" class="px-1 pt-1.5 pb-0.5 inline w-fit !h-fit max-w-full text-sm text-wrap line-clamp-1 text-ellipsis" title={file.name || file.path.split(sep()).pop()} onclick={() => handleAudioTrackSelect(file)}>
                                    {file.name || file.path.split(sep()).pop()}
                                </Button>
                                <a href="##" class="text-xs text-muted-foreground hover:text-foreground/80" onclick={() => handleArtistSelect(file)}>{file.artist || "Unknown Artist"}</a>

                            </div>

                        </li>
                    {/if}
                {/each}
            </ul>
        {:else}
            <!-- No audio files found -->
        {/if}
        
    {:catch error}
        {console.error(error)}
        <p>Error loading audio files: {error.message}</p>
    {/await}

</main>
