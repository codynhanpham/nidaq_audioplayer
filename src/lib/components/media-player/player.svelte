<script lang="ts" module>
    export type MediaPlayerProps = {
        class?: string;
    };
</script>


<script lang="ts">
	import "./style.css";
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { onMount } from 'svelte';
	import { cn } from '$lib/utils.js';

	import * as Avatar from "$lib/components/ui/avatar/index.js";
	import { Button, buttonVariants } from '$lib/components/ui/button/index.js';
	import * as HoverCard from "$lib/components/ui/hover-card/index.js";
	import * as Select from "$lib/components/ui/select/index.js";
    import { Slider } from "$lib/components/ui/slider/index.js";

	import {
		Disc3,
		Play,
		Pause,
		Repeat,
		Repeat1,
		SkipBack,
		SkipForward,
		TableOfContents,
		Volume1,
		Volume2,
		VolumeX,
	} from '@lucide/svelte/icons';

	import { MediaPlayerData } from './playerData.svelte';
	import { wsSendOnce } from "./tasks/wsconnection.svelte";

	let {
		class: className
	}: MediaPlayerProps = $props();

	let manualSeek: {
		active: boolean;
		time: number | undefined;
		committed: boolean;
	} = $state({
		active: false,
		time: undefined,
		committed: false
	});

	// Throttled slider value calculation
	let throttledSliderValue = $state(0);
	let rafId: number | null = null;

	/**
	 * Calculate the slider value based on progress and duration.
	 * Converts percentage progress to a millisecond-based value.
	 * @param progress Current playback progress as a percentage [0-100], null if no media loaded
	 * @param duration Total duration of the media in seconds, null if not applicable or media not loaded
	 * @return Calculated slider value in milliseconds, 0 if no media loaded or duration is 0
	 */
	function calcSliderValue(progress: number | null, duration: number | null): number {
		if (progress === null || duration === null || duration === 0) return 0;
		// Convert percentage progress to millisecond-based value
		return Math.round((progress / 100) * duration * 1000); // Convert seconds to milliseconds
	}

	// Update throttled slider value on next animation frame
	$effect(() => {
		const newValue = calcSliderValue(MediaPlayerData.progress, MediaPlayerData.duration);
		
		if (rafId) {
			cancelAnimationFrame(rafId);
		}
		
		rafId = requestAnimationFrame(() => {
			throttledSliderValue = newValue;
			rafId = null;
		});
	});

	/**
	 * Calculate the duration string in H:M:SS.s format
	 * @param duration Total duration of the media in seconds, if null, default to 0:00
	 * @return Formatted duration string in H:M:SS.s format
	 */
	function formatDuration(duration: number | null): string {
		if (duration === null) return '0:00.0';
		// Calc H:M:SS
		const hours = Math.floor(duration / 3600);
		const minutes = Math.floor((duration % 3600) / 60);
		// const seconds = duration % 60;
		// Round seconds to 1 decimal place
		const seconds = Math.floor(duration % 60);
		const milliseconds = Math.floor((duration % 1) * 1000);
		// return `${hours > 0 ? hours + ':' : ''}${minutes}:${seconds < 10 ? '0' : ''}${seconds}`;
		let result = '';
		if (hours > 0) {
			result += hours + ':';
		}
		if (hours > 0 && minutes < 10) {
			result += '0';
		}
		result += minutes + ':';
		result += seconds < 10 ? '0' + seconds : seconds;
		result += '.' + Math.floor(milliseconds / 100);
		return result;
	}
	/**
	 * Calculate and format the current progress (given a total duration) in H:M:SS format
	 * @param progress Current playback progress as a percentage [0-100], null if no media loaded
	 * @param duration Total duration of the media in seconds, null if not applicable or media not loaded
	 * @return Formatted progress string in H:M:SS format
	 */
	function formatProgress(progress: number | null, duration: number | null): string {
		if (progress === null || duration === null || duration === 0) return '0:00.0';
		// Calculate current time in seconds
		const currentTime = (progress / 100) * duration;
		return formatDuration(currentTime);
	}

	function cycleMediaDataLoopModes() {
		const currentLoopMode = MediaPlayerData.loop;
		const loopModes = ["none", "all", "one"];
		const nextLoopMode = loopModes[(loopModes.indexOf(currentLoopMode) + 1) % loopModes.length];
		MediaPlayerData.loop = nextLoopMode as "none" | "all" | "one";
	}
	function currentLoopModeTitle() {
		switch (MediaPlayerData.loop) {
			case "none":
				return "No Repeat";
			case "all":
				return "Repeat All";
			case "one":
				return "Repeat One";
		}
	}

	/**
	 * Get the current chapter title based on the current progress and chapters.
	 * If no chapters are available, returns null.
	 * Otherwise, return value as `{index}__{timestamp}__{title}` to avoid ambiguity.
	 * @return Current chapter title or null if no chapters are available
	 */
	function getCurrentChapter(): string | null {
		const chapters = MediaPlayerData.audioInfo?.chapters;
		if (!chapters?.length || !MediaPlayerData.duration) return null;
		
		// If at the beginning, return first chapter
		if (!MediaPlayerData.progress) {
			const chapter = chapters[0];
			return `0__${chapter.timestamp}__${chapter.title}`;
		}

		const currentTime = (MediaPlayerData.progress / 100) * MediaPlayerData.duration;
		const nextChapterIndex = chapters.findIndex(ch => ch.timestamp > currentTime);
		
		// If no next chapter found, we're in the last chapter
		if (nextChapterIndex === -1) {
			const lastIndex = chapters.length - 1;
			const chapter = chapters[lastIndex];
			return `${lastIndex}__${chapter.timestamp}__${chapter.title}`;
		}
		
		// If first chapter is ahead of current time, we're before any chapter
		if (nextChapterIndex === 0) return null;
		
		// Otherwise, we're in the chapter before the next one
		const chapterIndex = nextChapterIndex - 1;
		const chapter = chapters[chapterIndex];
		return `${chapterIndex}__${chapter.timestamp}__${chapter.title}`;
	}
	let currentChapter: string | undefined = $derived.by(() => {
		return getCurrentChapter() || undefined;
	});

	function seekToChapter(timestamp: number) {
		if (MediaPlayerData.duration === null) return;
		wsSendOnce({
			task: "seek",
			data: {
				time: timestamp
			}
		});
	}

	function togglePlayPause() {
		if (MediaPlayerData.isPlaying) {
			wsSendOnce({
				task: "pause"
			})
		} else {
			if (MediaPlayerData.playbackCompleted) {
				wsSendOnce({
					task: "seek",
					data: {
						time: 0
					}
				});
			}
			wsSendOnce({
				task: "play"
			})
		}
	}

	function toggleMute() {
		MediaPlayerData.muted = !MediaPlayerData.muted;
		wsSendOnce({
			task: "volume",
			data: {
				volume: MediaPlayerData.muted ? 0 : MediaPlayerData.volume
			}
		});
	}

	function syncPlayerVolume() {
		wsSendOnce({
			task: "volume",
			data: {
				volume: MediaPlayerData.volume
			}
		});
	}

	// Spacebar toggles play/pause
	const handleKeyDown = (event: KeyboardEvent) => {
		if (event.code === 'Space') {
			event.preventDefault();
			togglePlayPause();
		}
	};

	onMount(() => {
		window.addEventListener('keydown', handleKeyDown);

		const sliderElem = document.querySelector('[data-progress-slider]');
		sliderElem?.addEventListener('pointerdown', () => {
			manualSeek.active = true;
			manualSeek.committed = false;
			MediaPlayerData.pauseAutomaticWsProgressUpdate = true;
		});
		sliderElem?.addEventListener('pointerup', () => {
			manualSeek.active = false;
			manualSeek.committed = true;
			MediaPlayerData.pauseAutomaticWsProgressUpdate = false;
			if (manualSeek.time && MediaPlayerData.duration) {
				MediaPlayerData.progress = (manualSeek.time / MediaPlayerData.duration) * 100;
			}
		});

		// Just in case the user releases the pointer outside the slider
		window.addEventListener('pointerup', () => {
			if (manualSeek.active) {
				manualSeek.active = false;
				manualSeek.committed = true;
				MediaPlayerData.pauseAutomaticWsProgressUpdate = false;
				if (manualSeek.time && MediaPlayerData.duration) {
					MediaPlayerData.progress = (manualSeek.time / MediaPlayerData.duration) * 100;
				}
			}
		});

		return () => {
			if (rafId) {
				cancelAnimationFrame(rafId);
			}
			window.removeEventListener('keydown', handleKeyDown);
			sliderElem?.removeEventListener('pointerdown', () => {
				manualSeek.active = false;
				MediaPlayerData.pauseAutomaticWsProgressUpdate = false;
			});
		};
	});
</script>


<section class={cn(
		'mediaplayer-override isolate',
		'bg-background/40',
		// 'bg-background/70 backdrop-blur-3xl',
		'pointer-events-auto fixed left-0 bottom-6 z-[99999]',
		className,
		((MediaPlayerData.audioInfo?.chapters && MediaPlayerData.audioInfo.chapters.length > 0) ? 'h-21' : 'h-19'),
		'w-full sm:h-16.5',
	)}
>
    <div class="group/player relative w-full h-full flex items-center justify-between px-1.5 sm:px-3 pt-1">
        <div
			data-progress-slider-wrapper
			class="absolute left-0 -top-1 w-full h-fit"
		>
			<Slider
				type="single"
				data-progress-slider
				thumbPositioning={"exact"}
				class="group/slider py-1 w-[calc(100%-var(--spacing)*2)] mx-auto [&_[data-slot='slider-track']]:!cursor-pointer [&_[data-slot='slider-track']]:!mask-linear-[270deg,transparent_-8em,black_calc(var(--spacing)*4+1.5%)),black_calc(100%-var(--spacing)*4-1.5%)),transparent_calc(100%+8em)] [&_[data-slot='slider-track']]:!mask-r-from-99% [&_[data-slot='slider-track']]:duration-200 [&_[data-slot='slider-track']]:!h-0.5 [&_[data-slot='slider-track']]:group-hover/player:!h-1 [&_[data-slot='slider-track']]:group-active/player:!h-1 [&_[data-slot='slider-track']]:group-focus-within/player:!h-1 [&_[data-slot='slider-range']]:!bg-primary/90 [&_[data-slot='slider-range']]:!backdrop-blur-3xl [&_[data-slot='slider-thumb']]:!size-3 [&_[data-slot='slider-thumb']]:!p-1 [&_[data-slot='slider-thumb']]:!bg-primary [&_[data-slot='slider-thumb']]:!border-none [&_[data-slot='slider-thumb']]:!opacity-0 [&_[data-slot='slider-thumb']]:group-hover/player:!opacity-100 [&_[data-slot='slider-thumb']]:group-active/player:!opacity-100 [&_[data-slot='slider-thumb']]:group-focus-within/player:!opacity-100 [&_[data-slot='slider-thumb']]:focus-visible:!opacity-100 [&_[data-slot='slider-thumb']]:!transition-opacity [&_[data-slot='slider-thumb']]:!duration-200"
				value={throttledSliderValue}
				min={0}
				max={Math.ceil((MediaPlayerData.duration || 0) * 1000)}
				step={20}
				onValueChange={(value) => {
					const seek_to_progress = value / (Math.ceil((MediaPlayerData.duration || 0) * 1000)) * 100;
					MediaPlayerData.progress = seek_to_progress;
					
					// Handle manual seeking while media is playing
					if (MediaPlayerData.duration === null) return;

					if (manualSeek.active && !manualSeek.committed) {
						manualSeek.time = (seek_to_progress / 100) * (MediaPlayerData.duration);
					}

					if (manualSeek.committed) {
						const seek_to_seconds = manualSeek.time;
						wsSendOnce({
							task: "seek",
							data: {
								time: seek_to_seconds
							}
						});
						manualSeek.active = false;
						manualSeek.committed = false;
						manualSeek.time = undefined;
						MediaPlayerData.pauseAutomaticWsProgressUpdate = false;
					}
				}}
				onValueCommit={(value) => {
					if (MediaPlayerData.duration === null || MediaPlayerData.isPlaying) return;
					const seek_to_progress = value / (Math.ceil((MediaPlayerData.duration || 0) * 1000)) * 100;
					const seek_to_seconds = (seek_to_progress / 100) * (MediaPlayerData.duration);
					manualSeek.active = false; manualSeek.committed = false; manualSeek.time = undefined;
					MediaPlayerData.pauseAutomaticWsProgressUpdate = false;
					wsSendOnce({
						task: "seek",
						data: {
							time: seek_to_seconds
						}
					});
				}}
			/>
		</div>

		<div class="w-full h-full flex flex-col items-center justify-center">
			<div data-xs-player-timing class="w-full max-h-fit flex sm:hidden text-xs text-muted-foreground items-center justify-between gap-3 px-2.5 pt-2 [&_span]:mb-0">
				<span class="shrink-0">{formatProgress(MediaPlayerData.progress, MediaPlayerData.duration)}</span>
				{#if MediaPlayerData.audioInfo?.chapters}
					<!-- <Button variant="ghost" class="shrink !pt-0.5 !pb-1 !px-2 !h-fit" onclick={() => {}} aria-label="Chapters" title="Chapters">
						<span class="text-wrap text-xs text-muted-foreground font-bold max-w-full line-clamp-1 text-ellipsis">{getCurrentChapter()?.split('__')[2]}</span>
					</Button> -->

					<Select.Root type="single" bind:value={currentChapter} disabled={!MediaPlayerData.audioInfo?.chapters || MediaPlayerData.audioInfo.chapters.length === 0}>
						<Select.Trigger class={"border-none shrink !pt-0.5 !pb-0.5 !px-2.5 !h-fit"} aria-label="Chapters" title="Chapters">
							<!-- <TableOfContents class={"size-4 text-foreground"} /> -->
							<span class="text-wrap text-xs text-muted-foreground font-bold max-w-full line-clamp-1 text-ellipsis">{getCurrentChapter()?.split('__')[2]}</span>
						</Select.Trigger>
						<Select.Content class="z-[99999]" side="top" sideOffset={-1} collisionPadding={{top: 42}}>
							<Select.SelectGroup>
								<Select.Label class="py-1 px-1.5">Chapters</Select.Label>
								{#if MediaPlayerData.audioInfo?.chapters && MediaPlayerData.audioInfo?.chapters.length > 0}
									{#each MediaPlayerData.audioInfo.chapters as chapter, i}
										<Select.Item value={`${i}__${chapter.timestamp}__${chapter.title}`} class="px-2.5"
											onclick={() => { seekToChapter(chapter.timestamp); }}
										>
											<div class="w-full flex items-center justify-between gap-4 mr-6">
												<span class="text-xs text-left text-muted-foreground">{formatDuration(chapter.timestamp)}</span>
												<span class="max-w-[24ch] flex-1 text-left line-clamp-1 text-ellipsis">{chapter.title}</span>
											</div>
										</Select.Item>
									{/each}
								{/if}
							</Select.SelectGroup>
						</Select.Content>
					</Select.Root>

				{/if}
				
				<span class="shrink-0">{formatDuration(MediaPlayerData.duration)}</span>
			</div>

			<div class="w-full flex-1 flex items-center justify-between gap-8 md:gap-10">
				<section data-player-controls-group class="flex flex-col py-0 sm:flex-row items-center justify-center gap-2">
					<div data-player-controls class="flex items-center justify-center gap-0.5">
						<Button variant="ghost" class="media-button" onclick={() => {}} aria-label="Skip Back">
							<SkipBack class="size-4" />
						</Button>
						<Button variant="ghost" class="media-button" onclick={() => { togglePlayPause() }} aria-label={MediaPlayerData.isPlaying ? "Pause" : "Play"}>
							{#if MediaPlayerData.isPlaying}
								<Pause class="size-4" />
							{:else}
								<Play class="size-4" />
							{/if}
						</Button>
						<Button variant="ghost" class="media-button" onclick={() => {}} aria-label="Skip Forward">
							<SkipForward class="size-4" />
						</Button>
					</div>
					<div data-player-timing-1 class="max-h-fit text-sm lg:text-base text-muted-foreground hidden sm:flex items-center justify-center gap-1.5">
						<span>{formatProgress(MediaPlayerData.progress, MediaPlayerData.duration)}</span>
						<div class="hidden sm:flex items-center justify-center gap-1.5">
							<span>/</span>
							<span>{formatDuration(MediaPlayerData.duration)}</span>
						</div>
					</div>
				</section>

				<section data-player-media-info class="flex flex-1 grow items-center justify-center gap-4 sm:gap-5">
					{#if MediaPlayerData.audioInfo}
						<div class="[&_div]:!rounded-sm [&_span]:!rounded-sm">
							{#if MediaPlayerData.audioInfo?.thumbnail}
								<Avatar.Root class="size-8 sm:size-9">
									<Avatar.Image class="h-full aspect-square" src={MediaPlayerData.audioInfo.thumbnail} alt="@shadcn" />
									<Avatar.Fallback class="h-full aspect-square">
										<Disc3 class={cn("size-4.5 sm:size-5", MediaPlayerData.isPlaying ? "animate-spin" : "")} />
									</Avatar.Fallback>
								</Avatar.Root>
							{:else}
								<Avatar.Root class="size-8 sm:size-9">
									<Avatar.Image class="h-full aspect-square" src="##" alt="@shadcn" />
									<Avatar.Fallback class="h-full aspect-square">
										<Disc3 class={cn("size-4.5 sm:size-5", MediaPlayerData.isPlaying ? "animate-spin" : "")} />
									</Avatar.Fallback>
								</Avatar.Root>
							{/if}
						</div>

						<div class="max-w-full flex flex-col items-center justify-center gap-0">
							<span class="mb-0 text-left text-sm text sm:text-base max-w-full text-ellipsis line-clamp-1" title={MediaPlayerData.audioInfo?.name || ""}>{MediaPlayerData.audioInfo?.name || ""}</span>
							<span class="text-xs sm:text-sm text-center text-muted-foreground max-w-full text-ellipsis line-clamp-1" title={MediaPlayerData.audioInfo?.artist || ""}>{MediaPlayerData.audioInfo?.artist || ""}</span>
						</div>
					{/if}
				</section>

				<section data-player-options class="flex items-center justify-center gap-0.5">
					<div class="hidden sm:block">
						<Select.Root type="single" bind:value={currentChapter} disabled={!MediaPlayerData.audioInfo?.chapters || MediaPlayerData.audioInfo.chapters.length === 0}>
							<Select.Trigger class={"!bg-transparent media-button border-none"} aria-label="Chapters" title="Chapters">
								<TableOfContents class={"size-4 text-foreground"} />
							</Select.Trigger>
							<Select.Content class="z-[99999] bg-background/85 backdrop-blur-2xl" side="top" sideOffset={14} collisionPadding={{top: 42, right: 4}}>
								<Select.SelectGroup>
									<Select.Label class="py-1 px-1.5">Chapters</Select.Label>
									{#if MediaPlayerData.audioInfo?.chapters && MediaPlayerData.audioInfo?.chapters.length > 0}
										{#each MediaPlayerData.audioInfo.chapters as chapter, i}
											<Select.Item value={`${i}__${chapter.timestamp}__${chapter.title}`} class="px-2.5"
												onclick={() => { seekToChapter(chapter.timestamp); }}
											>
												<div class="w-full flex items-center justify-between gap-4 mr-6">
													<span class="text-xs text-left text-muted-foreground">{formatDuration(chapter.timestamp)}</span>
													<span class="max-w-[24ch] flex-1 text-left line-clamp-1 text-ellipsis">{chapter.title}</span>
												</div>
											</Select.Item>
										{/each}
									{/if}
								</Select.SelectGroup>
							</Select.Content>
						</Select.Root>
					</div>
					<div>
						<Button variant="ghost" class="media-button" onclick={() => { cycleMediaDataLoopModes(); }} aria-label={currentLoopModeTitle()} title={currentLoopModeTitle()}>
							{#if MediaPlayerData.loop === "none"}
								<Repeat class="size-4 text-muted-foreground/80" />
							{:else if MediaPlayerData.loop === "all"}
								<Repeat class="size-4 text-primary" />
							{:else if MediaPlayerData.loop === "one"}
								<Repeat1 class="size-4 text-primary" />
							{/if}
						</Button>
					</div>
					<div>
						<HoverCard.Root
							openDelay={50}
							closeDelay={0}
						>
							<HoverCard.Trigger type="button" class={cn(buttonVariants({ variant: "ghost", size: "icon" }), "media-button")} aria-label="Volume Control" title="Volume Control" onclick={() => { toggleMute() }}>
								{#if MediaPlayerData.muted || MediaPlayerData.volume === 0}
									<VolumeX class="size-4" />
								{:else if MediaPlayerData.volume >= 60}
									<Volume2 class="size-4" />
								{:else}
									<Volume1 class="size-4" />
								{/if}
							</HoverCard.Trigger>
							<HoverCard.Content class="z-[99999] bg-background/70 backdrop-blur-2xl w-fit h-fit px-2 pt-2 pb-4.5 flex flex-col items-center justify-start" side="top" sideOffset={-1} collisionPadding={{top: 42, right: 4}}>
								<span class="text-[9px] text-primary/90 font-semibold select-none pointer-events-none mb-2 p-0 w-[4.5ch] text-center">{MediaPlayerData.volume}%</span>
								<Slider type="single" orientation="vertical" bind:value={MediaPlayerData.volume} min={0} max={100} step={1} thumbPositioning="exact" class="!max-h-38 !min-h-20 [@media(max-height:360px)]:!min-h-16 !min-w-1 [&_[data-slot='slider-track']]:!bg-muted-foreground/70 [&_[data-slot='slider-track']]:!w-1 [&_[data-slot='slider-thumb']]:!size-3 [&_[data-slot='slider-thumb']]:!p-1 [&_[data-slot='slider-thumb']]:!bg-primary"
								onValueChange={() => { 
									syncPlayerVolume();
								}}
								/>
							</HoverCard.Content>
						</HoverCard.Root>
					</div>
				</section>
			</div>

		</div>
    </div>
    
</section>