<script lang="ts" module>
    import { libraryStore, type Library, rescanLibrary, getLastStoreUpdated, getLastLibbinHash, setLastLibbinHash } from "./components/index.js";
    (async () => {
        rescanLibrary(await libraryStore);
    })();
    import { page } from "$app/state";

    let audioFilesList: Promise<string[]> = $derived.by(async () => {
        getLastStoreUpdated(); // Force update from store when it's updated
        const store = await libraryStore;
		const libraryData = await store.get<Library>('library');
		if (!libraryData) return [];
		return libraryData.audioFiles;
	});

    async function getMediaInfo(file: string, dropExtras: boolean = true): Promise<AudioInfo | undefined> {
        try {
            const metadata = await invoke("get_media_metadata", { path: file }) as AudioInfo & {extras?: Record<string, any>} | undefined;

            // console.log(metadata);

            // Should drop extras from metadata most of the time since it will eat up memory
            // (extra metadata like lyrics or additional artwork)
            // Only set dropExtras=false if you really want to list all metadata from the file
            if (metadata && metadata.extras && dropExtras) {
                delete metadata.extras;
            }
            return metadata;
        }
        catch {
            console.error("Failed to retrieve media info for:", file);
            return undefined;
        }
    }

    let audioFiles: (AudioInfo | undefined)[] | null = $state(null);

    /** Update audioFiles state without blocking */
    export async function refreshAudioMetadata(rescan: boolean = false, writefile: boolean = false) {
        const files = await audioFilesList;
        if (!files) return null;

        // The batch size to update UI and/or parse metadata
        const batchSize = 20;

        // First, check browser cache for existing data
        // ...

        // Try the cached bin file next
        const libbinPath = `${await appDataDir()}/library.bin`;
        if (!rescan) {
            try {
                const cachedBinData = await invoke("load_audio_metadata", { inputPath: libbinPath }) as AudioInfo[] | undefined;
                if (cachedBinData) {
                    // Set cache here ...

                    audioFiles = cachedBinData;
                    return;
                }
            }
            catch (error) {
                // This is fine, gonna try a different strategy next
            }
        }

        // If all else fails, walk the dir and load the metadata from scratch
        
        // Initialize audioFiles as empty array for progressive updates
        audioFiles = [];
        
        const allMetadata: (AudioInfo | undefined)[] = [];
        
        for (let i = 0; i < files.length; i += batchSize) {
            const batch = files.slice(i, i + batchSize);
            const batchResults = await Promise.all(batch.map(file => getMediaInfo(file, true)));
            
            // Add batch results to the accumulated metadata
            allMetadata.push(...batchResults);
            
            // Filter and sort current accumulated results
            const filteredContent = allMetadata.filter((item): item is AudioInfo => item !== undefined);
            filteredContent.sort((a, b) => {
                const nameA = a?.name.toLowerCase() || "";
                const nameB = b?.name.toLowerCase() || "";
                return nameA.localeCompare(nameB);
            });
            
            // Update audioFiles state to trigger UI refresh
            audioFiles = [...filteredContent];
        }

        const finalFilteredContent = allMetadata.filter((item): item is AudioInfo => item !== undefined);
        finalFilteredContent.sort((a, b) => {
            const nameA = a?.name.toLowerCase() || "";
            const nameB = b?.name.toLowerCase() || "";
            return nameA.localeCompare(nameB);
        });

        
        audioFiles = finalFilteredContent;

        if (!writefile) {
            return;
        }

        // Save the final metadata to cache file

        const store = await libraryStore;
        const lastLibbinHash = await getLastLibbinHash(store);
        let metadataHash: string | undefined;
        if (!lastLibbinHash) {
            invoke("save_audio_metadata", { audioMetadata: finalFilteredContent, outputPath: libbinPath });
            metadataHash = await invoke("calculate_audio_metadata_hash", { audioMetadata: finalFilteredContent }) as string;
            setLastLibbinHash(store, metadataHash);
            return;
        }

        if (!metadataHash) {
            metadataHash = await invoke("calculate_audio_metadata_hash", { audioMetadata: finalFilteredContent }) as string;
        }
        if (metadataHash !== lastLibbinHash) {
            invoke("save_audio_metadata", { audioMetadata: finalFilteredContent, outputPath: libbinPath });
            setLastLibbinHash(store, metadataHash);
        }
    }

    let lastAudioFiles: (AudioInfo | undefined)[] | null = null;
    let fully_rendered_once = false; // this flag determines whether must refresh audio metadata on load again

    // Initial load
    refreshAudioMetadata(true).then(() => {
        fully_rendered_once = true; // Set this to true after the first load
        if (page.url.pathname === '/library') {
            return;
        }

        // Handle the case where this is triggered while the user is not yet at this page
        // (during prefetching): should do what onDestroy does
        audioFilesList.then((data) => {
            const numfiles = data.length;
            if (lastAudioFiles === null && audioFiles?.length === numfiles) {
                lastAudioFiles = audioFiles;
                audioFiles = null; // Clear the audioFiles state to free up memory
            }
        });
    });

</script>

<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";

	import { getCurrentWindow } from '@tauri-apps/api/window';
    import { sep, appDataDir } from '@tauri-apps/api/path';

    import { Button } from '$lib/components/ui/button/index.js'
    import { Skeleton } from "$lib/components/ui/skeleton/index.js";

    import { getCurrentWebview } from "@tauri-apps/api/webview";
    import type { UnlistenFn } from "@tauri-apps/api/event";
    import { onMount, onDestroy } from "svelte";
    import { beforeNavigate } from "$app/navigation";

    import {
        FileMusic
    } from "@lucide/svelte/icons";
    
    import LibrarySelector from './components/library-selector.svelte';
    import { type AudioInfo, MediaPlayerData } from "$lib/components/media-player/playerData.svelte.js";
	import { wsSendOnce } from '@components/media-player/tasks/wsconnection.svelte';
    import { StatusBarData } from "@components/app-statusbar";



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

        if (MediaPlayerData.isPlaying) {
            wsSendOnce({
                task: "pause"
            });
        }

        wsSendOnce({
            task: "load_audio",
            data: {
                device_name: StatusBarData.niDaqSelectedDevice?.name,
                file_path: data.path,
                ao_channels: ['/ao0', '/ao1'],
                ai_channels: ['/ai0', '/ai1'],
                do_channels: ['/port0/line0', '/port0/line1'],
                volume: MediaPlayerData.volume,
                samples_per_frame: 8192,
                flip_lr_stereo: MediaPlayerData.flipLRStereo,
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

    // Crazy workaround to speed up rendering:
    // On destroy, save the data to a non-reactive variable and keep it there
    // On page load, since the reactive audioFiles is empty, Svelte won't spend ages to render before responding to navigation clicks
    // Page transition thus happens immediately, then only onMount that audioFiles is restored and the page is rendered
    onDestroy(() => {
        audioFilesList.then((data) => {
            const numfiles = data.length;
            if (lastAudioFiles === null && audioFiles?.length === numfiles) {
                lastAudioFiles = audioFiles;
                audioFiles = null; // Clear the audioFiles state to free up memory
            }
        });
    });

    // Handle user nav while page still rendering
    let stop_rendering = $state(false);
    beforeNavigate(() => {
        stop_rendering = true; // this breaks the main rendering loop in onMount
        // flush audioFiles back to lastAudioFiles
        if (audioFiles && audioFiles.length > 0) {
            lastAudioFiles?.push(...audioFiles);
            audioFiles = null; // Clear the audioFiles state to free up memory
        }
    });

    onMount(() => {
        // This load audioFiles back from lastAudioFiles
        if (lastAudioFiles && fully_rendered_once) {
            function breakTask() {
                return new Promise((resolve) => {
                    setTimeout(resolve, 0);
                });
            }
            audioFiles = [];
            (async () => {
                for (let i = 0; i < lastAudioFiles.length; i++) {
                    const file = lastAudioFiles[i];
                    if (file) {
                        audioFiles.push(file);
                        
                        // render the first 50 items immediately. render the rest once every 100. this also blocks page navigation as a bonus, so that audioFiles and lastAudioFiles get reset properly in between page loads
                        if (i <= 50) {
                            audioFiles = audioFiles; // kinda strange, but reassigning the $state var will update the #each block
                            await breakTask();
                        } else {
                            if (i % 100 === 0) {
                                audioFiles = audioFiles;
                                await breakTask(); // wait for the next event loop to render the next batch
                            }
                        }
                    }
                    if (stop_rendering) {
                        stop_rendering = false;
                        break;
                    }
                }
                audioFiles = audioFiles; // render the rest
                lastAudioFiles = null;
            })();
        }
        else {
            refreshAudioMetadata(true).then(() => {
                fully_rendered_once = true;
            });
        }
    });

</script>

<main class="w-full h-full min-h-fit space-y-2 p-2">
    <LibrarySelector />

    {#if audioFiles === null}
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
    {:else if audioFiles && audioFiles.length > 0}
        <!-- Display loaded audio files -->
        <ul class="grid gap-x-4 gap-y-4 grid-cols-[repeat(auto-fill,minmax(9.5rem,1fr))] md:grid-cols-[repeat(auto-fill,minmax(10rem,1fr))] my-3 mx-2">
            {#each audioFiles as file, i (file?.path)}
                {#if file}
                    <!-- <li>{file.name || file.path.split("\\").pop()}</li> -->
                    <li class="w-full h-full flex flex-col items-center justify-start">
                        {#if file.thumbnail}
                            <Button variant="ghost" class="p-0 w-full h-fit rounded-sm overflow-hidden" title={file.name || file.path.split(sep()).pop()} onclick={() => handleAudioTrackSelect(file)}>
                                <img src={file.thumbnail} alt={`${file.name || file.path.split(sep()).pop()} cover image`} class="w-full h-fit aspect-square object-cover" />
                            </Button>
                        {:else}
                            <Button variant="ghost" class="w-full h-fit aspect-square flex items-center justify-center bg-muted rounded-sm" title={file.name || file.path.split(sep()).pop()} onclick={() => handleAudioTrackSelect(file)}>
                                <FileMusic class="size-1/3" />
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
        <!-- No audio files found, just keep the skeleton state for now -->
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
    {/if}
</main>