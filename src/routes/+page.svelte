<script lang="ts">
	import { invoke } from "@tauri-apps/api/core";
    import { sep, appDataDir } from '@tauri-apps/api/path';

	import { Button } from '$lib/components/ui/button/index.js'

	import {
		History,
		FileMusic,
		TableOfContents,
	} from '@lucide/svelte/icons';

	import { NavigationRoutes } from '$lib/components/app-navigator';
	import { page } from "$app/state";

	import { type AudioInfo } from "$lib/components/media-player/playerData.svelte.js";
	import { HistoryAudioInfo, type HistoryData, historyStore, loadHistoryFromStore } from "./history/history.svelte";
	import { handleArtistSelect, handleAudioTrackSelect, formatChannelCount } from "./library/+page.svelte";
	import { formatDuration } from "@components/media-player/player.svelte";
    import { onMount } from "svelte";


	// Get the last 6 last played items from the database
	let recentlyPlayed: Promise<AudioInfo[]> = $derived.by(async () => {
		if (!HistoryAudioInfo.audioInfo || HistoryAudioInfo.audioInfo.length === 0) {
			// Load history from the database if it's not already loaded
			await loadHistoryFromStore(await historyStore);
		}
		const historyData: AudioInfo[] = HistoryAudioInfo.audioInfo || [];
		// Take last 6 items and reverse to show the most recent first
		return historyData.slice(-6).reverse();
	});

	onMount(() => {
		(async () => {
			if (!HistoryAudioInfo.audioInfo || HistoryAudioInfo.audioInfo.length === 0) {
				// Load history from the database if it's not already loaded
				await loadHistoryFromStore(await historyStore);
			}
			// Log the recently played items for debugging
			console.log("Recently Played Items:", recentlyPlayed);
		})();
	});

</script>

<main class="w-full h-full min-h-fit space-y-2 p-2">
	<!-- NAVIGATION -->
	<section class="w-full p-3 bg-muted/40 border border-border/50 rounded-lg">
		<h2 class="mx-2 text-lg font-semibold">Navigation</h2>
		<div class="grid gap-x-2 gap-y-2 grid-cols-[repeat(auto-fill,minmax(9rem,1fr))] md:grid-cols-[repeat(auto-fill,minmax(12rem,1fr))] my-3 mx-2">
			{#each NavigationRoutes as route, i}
				{#if route.url !== "/" && route.url !== page.url.pathname}
					<Button
						variant="outline"
						class="group w-full h-fit flex flex-row items-center justify-start gap-3 px-4 py-2 rounded-lg text-left hover:bg-accent/50 focus-visible:bg-accent/50"
						href={route.url}
						aria-label={route.name}
						title={route.name}
					>
						<div class="bg-muted/30 rounded-lg">
							<route.icon class="size-4 md:size-5 text-foreground/80 group-hover:text-accent-foreground" />
						</div>
						<div class="w-full space-y-1">
							<span class="w-full text-foreground/80 group-hover:text-accent-foreground text-wrap line-clamp-1 text-ellipsis">{route.name}</span>
						</div>
					</Button>
				{/if}
			{/each}
		</div>
	</section>


	<!-- RECENTLY PLAYED -->
	<section class="w-full p-3 bg-muted/10 border border-border/80 rounded-lg">
		<h2 class="mx-2 text-lg font-semibold">Recently Played</h2>
		<div class="space-y-1 my-3 mx-2">
			{#await recentlyPlayed}
				<Button
					variant="ghost"
					class="group h-fit font-normal flex flex-row items-center justify-start gap-3 px-4 py-2 rounded-lg text-left border border-border/90 hover:border-border/100 focus-visible:border-border hover:bg-accent/20 focus-visible:bg-accent/30"
					href="/library/album/12345"
					aria-label="No Recently Played Items"
					title="No Recently Played Items"
					disabled
				>
					<div class="w-fit mx-auto flex items-center justify-center gap-2">
						<History class="size-4 text-foreground/90 group-hover:text-accent-foreground" />
						<span class="text-foreground/90 group-hover:text-accent-foreground">No recently played items</span>
					</div>
				</Button>
			{:then recentlyPlayed} 
				{#if recentlyPlayed.length === 0}
					<Button
						variant="ghost"
						class="group h-fit font-normal flex flex-row items-center justify-start gap-3 px-4 py-2 rounded-lg text-left border border-border/90 hover:border-border/100 focus-visible:border-border hover:bg-accent/20 focus-visible:bg-accent/30"
						href="/library/album/12345"
						aria-label="No Recently Played Items"
						title="No Recently Played Items"
						disabled
					>
						<div class="w-fit mx-auto flex items-center justify-center gap-2">
							<History class="size-4 text-foreground/90 group-hover:text-accent-foreground" />
							<span class="text-foreground/90 group-hover:text-accent-foreground">No recently played items</span>
						</div>
					</Button>
				{:else}
					{#each recentlyPlayed as file, i}
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
								<Button variant="link" class="w-full h-fit p-0 text-sm md:text-base text-left whitespace-normal break-all overflow-hidden line-clamp-2 hover:no-underline rounded-none" title={file.name || file.path.split(sep()).pop()} onclick={() => handleAudioTrackSelect(file)}>
                                    {file.name || file.path.split(sep()).pop()}
                                </Button>
                                <a href="##" class="mt-0.5 text-xs md:text-sm text-left text-muted-foreground hover:text-foreground/80 whitespace-normal break-all overflow-hidden line-clamp-2" title={file.artist || "Unknown Artist"} onclick={() => handleArtistSelect(file)}>{file.artist || "Unknown Artist"}</a>
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
					{/each}
				{/if}

			{:catch error}
				<Button
					variant="ghost"
					class="group h-fit font-normal flex flex-row items-center justify-start gap-3 px-4 py-2 rounded-lg text-left border border-border/90 hover:border-border/100 focus-visible:border-border hover:bg-accent/20 focus-visible:bg-accent/30"
					href="/library/album/12345"
					aria-label="Error loading recently played items"
					title="Error loading recently played items"
					disabled
				>
					<div class="w-fit mx-auto flex items-center justify-center gap-2">
						<History class="size-4 text-foreground/90 group-hover:text-accent-foreground" />
						<span class="text-foreground/90 group-hover:text-accent-foreground">Error loading recently played items</span>\
						<span class="sr-only text-left">Error details: {error.message}</span>
					</div>
				</Button>
			{/await}
		</div>
	</section>
</main>
