<script lang="ts" module>
    import { libraryStore, type Library, rescanLibrary, getLastStoreUpdated, getLastLibbinHash, setLastLibbinHash, listLibraryDirs } from "./components/index.js";
    (async () => {
        rescanLibrary(await libraryStore);
    })();

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

    function openLocationSelector() {
        if (!LibraryLocationSelector.display) {
            LibraryLocationSelector.display = true;
            LibraryLocationSelector.expanded = true;
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


    
    export function handleAudioTrackSelect(data: AudioInfo | undefined) {
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
                ao_channels: ['/ao0', '/ao1', '/ao2', '/ao3'], // On stereo file, backend will assign Left to ao0 and ao2, Right to ao1 and ao3
                ai_channels: [],
                do_channels: ['/port0/line0', '/port0/line1'],
                volume: MediaPlayerData.volume,
                samples_per_frame: 8192, // Slightly larger buffer to absolutely avoid underflow (especially for high compression files on slow computers)
                flip_lr_stereo: MediaPlayerData.flipLRStereo,
            }
        });
        wsSendOnce({
            task: "play"
        });

        addToHistory(data);
    }
    export function handleArtistSelect(data: AudioInfo | undefined) {
        if (!data) {
            return;
        }
    }


    export function formatChannelCount(count: number) {
        if (count === 1) return "Mono";
        if (count === 2) return "Stereo";
        if (count === 4) return "Quadraphonic";
        return `${count} Channels`;
    }



    // Initial load, pull from bin cache if possible, otherwise load from scratch (which will then save to cache for next time)
    refreshAudioMetadata(true, false).then(async () => {
        // If no audio file or no dir selected, then open the location selector to prompt user to select a dir
        const currentdirs = await listLibraryDirs(await libraryStore);
        if (!currentdirs || currentdirs.length === 0 || (!audioFiles || audioFiles.length === 0)) {
            openLocationSelector();
        }
    });

</script>

<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { sep, appDataDir } from '@tauri-apps/api/path';

    import { Button } from '$lib/components/ui/button/index.js'
    import { Skeleton } from "$lib/components/ui/skeleton/index.js";

    import VirtualList from '@humanspeak/svelte-virtual-list';

    import { onMount } from "svelte";

    import {
        FileMusic,
        TableOfContents,
    } from "@lucide/svelte/icons";
    
    import LibrarySelector from './components/library-selector.svelte';
    import { type AudioInfo, MediaPlayerData } from "$lib/components/media-player/playerData.svelte.js";
	import { wsSendOnce } from '@components/media-player/tasks/wsconnection.svelte';
    import { StatusBarData } from "@components/app-statusbar";
    import { addToHistory } from "../history/history.svelte.js";

	import { LibraryLocationSelector } from '@components/media-player/locationSelectorDisplay.svelte';
    import { formatDuration } from "@components/media-player/player.svelte";
    import { cn } from "$lib/utils";




    onMount(() => {
    });

</script>

<main class="w-full h-full min-h-fit space-y-2">
    <div class="w-full h-full flex flex-col items-start justify-start">
        {#if LibraryLocationSelector.display}
            <div class="w-full max-h-[60vh] shrink-0 overflow-y-auto p-2 pb-1" id="library-location-selector-container">
                <LibrarySelector />
            </div>
        {/if}

        {#if audioFiles === null}
            <VirtualList 
                items={Array.from({ length: 5 }, (_, i) => i)}
                viewportClass={cn("virtual-list-viewport", LibraryLocationSelector.display ? "mb-0.5" : "mb-0")}
            >
                {#snippet renderItem()}
                    <div class="w-full flex flex-row items-center justify-start my-2 px-2 gap-2">
                        <Skeleton class="p-0 size-20 sm:size-22 md:size-24 lg:size-26 aspect-square! rounded-sm" title={`Loading...`} />
                        <div class="w-full h-fit text-left flex flex-col items-start justify-start">
                            <Skeleton class="px-1 pt-1.5 pb-0.5 inline mt-0.5 w-3/4 h-5! max-w-full text-sm text-wrap line-clamp-1 text-ellipsis" />
                            <Skeleton class="text-xs text-muted-foreground mt-1 w-1/2 h-3.5! hover:text-foreground/80" />
                        </div>
                    </div>
                {/snippet}
            </VirtualList>
        {:else if audioFiles && audioFiles.length > 0}
            <VirtualList 
                items={audioFiles}
                viewportClass={cn("virtual-list-viewport", LibraryLocationSelector.display ? "mb-0.5" : "mb-0")}
            >
                {#snippet renderItem(file)}
                    {#if file}
                        <div class="w-full flex flex-row items-center justify-start my-0.5 sm:my-1 px-2 gap-2.5 sm:gap-3 md:gap-4 hover:bg-muted/50 rounded-lg first:mt-1.5 last:mb-1.5">
                            {#if file.thumbnail}
                                <Button variant="ghost" class="p-0 my-1.5 size-20 sm:size-22 md:size-24 lg:size-26 rounded-sm overflow-hidden" title={file.name || file.path.split(sep()).pop()} onclick={() => handleAudioTrackSelect(file)}>
                                    <img src={file.thumbnail} alt={`${file.name || file.path.split(sep()).pop()} cover image`} class="size-20 sm:size-22 md:size-24 lg:size-26 aspect-square object-cover" />
                                </Button>
                            {:else}
                                <Button variant="ghost" class="p-0 my-1.5 size-20 sm:size-22 md:size-24 lg:size-26 aspect-square flex items-center justify-center bg-muted rounded-sm" title={file.name || file.path.split(sep()).pop()} onclick={() => handleAudioTrackSelect(file)}>
                                    <FileMusic class="size-1/3" />
                                </Button>
                            {/if}
                            <div class="w-full h-fit flex flex-col items-start justify-start">
                                <Button variant="link" class="w-full h-fit p-0 text-sm md:text-base text-left whitespace-normal overflow-hidden line-clamp-2 hover:no-underline rounded-none" title={file.name || file.path.split(sep()).pop()} onclick={() => handleAudioTrackSelect(file)}>
                                    {file.name || file.path.split(sep()).pop()}
                                </Button>
                                <a href="##" class="mt-0.5 text-xs md:text-sm text-left text-muted-foreground hover:text-foreground/80 whitespace-normal overflow-hidden line-clamp-2" title={file.artist || "Unknown Artist"} onclick={() => handleArtistSelect(file)}>{file.artist || "Unknown Artist"}</a>
                                <span class="text-[0.6rem] sm:text-xs text-left text-muted-foreground mt-0.5 whitespace-normal overflow-hidden line-clamp-2">
                                    {file.sample_rate} Hz • {file.bit_depth}-bit • {formatChannelCount(file.channels)}
                                </span>
                            </div>
                            <div class="w-fit h-fit flex flex-col items-end justify-start">
                                <span class="mr-1 sm:ml-2 md:ml-4 text-xs md:text-sm text-muted-foreground whitespace-normal overflow-hidden line-clamp-2">{formatDuration(file.duration)}</span>
                                {#if file.chapters && file.chapters.length > 0}
                                    <div class="w-fit h-fit flex flex-row items-center justify-center">
                                        <span class="text-xs md:text-sm text-muted-foreground">{file.chapters.length}</span>
                                        <span class="sr-only lg:not-sr-only lg:inline text-sm lg:ml-1! text-muted-foreground">Chapters</span>
                                        <TableOfContents class="ml-1 size-3.5 sm:size-4 md:size-4.5 lg:hidden text-muted-foreground" />
                                    </div>
                                {/if}
                            </div>
                        </div>
                    {/if}
                {/snippet}
            </VirtualList>
        {:else}
            <!-- No audio files found, just keep the skeleton state for now -->
            <VirtualList 
                items={Array.from({ length: 5 }, (_, i) => i)}
                viewportClass={cn("virtual-list-viewport", LibraryLocationSelector.display ? "mb-0.5" : "mb-0")}
            >
                {#snippet renderItem()}
                    <div class="w-full flex flex-row items-center justify-start my-2 px-2 gap-2">
                        <Skeleton class="p-0 size-20 sm:size-22 md:size-24 lg:size-26 aspect-square! rounded-sm" title={`Loading...`} />
                        <div class="w-full h-fit text-left flex flex-col items-start justify-start">
                            <Skeleton class="px-1 pt-1.5 pb-0.5 inline mt-0.5 w-3/4 h-5! max-w-full text-sm text-wrap line-clamp-1 text-ellipsis" />
                            <Skeleton class="text-xs text-muted-foreground mt-1 w-1/2 h-3.5! hover:text-foreground/80" />
                        </div>
                    </div>
                {/snippet}
            </VirtualList>
        {/if}
    </div>
</main>