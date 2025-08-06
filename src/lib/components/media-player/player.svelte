<script lang="ts" module>
    export type MediaPlayerProps = {
        class?: string;
    };
</script>


<script lang="ts">
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { onMount } from 'svelte';
	import { cn } from '$lib/utils.js';

	import { Button } from '$lib/components/ui/button/index.js';
    
    import { Slider } from "$lib/components/ui/slider/index.js";

	import IconMinus from '@lucide/svelte/icons/minus';
	import { MediaPlayerData } from './playerData.svelte';

	let {
		class: className
	}: MediaPlayerProps = $props();


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


	onMount(async () => {
	});
</script>


<section class={cn(
		'bg-background/80 backdrop-blur-3xl pointer-events-auto fixed left-0 bottom-6 z-[99999]',
		className,
		'w-full h-18',
		// 'bg-blue-100'
	)}
>
    <div class="group/player relative w-full h-full flex items-center justify-between px-4">
        <div
			data-progress-slider-wrapper
			class="absolute left-0 -top-1 w-full h-fit"
		>
			<Slider
				type="single"
				data-progress-slider
				class="group/slider py-1 [&_[data-slot='slider-track']]:!cursor-pointer [&_[data-slot='slider-track']]:!mask-linear-[270deg,transparent,black_calc(var(--spacing)*1.5+0px)),black_calc(100%-var(--spacing)*1.5+0px)),transparent] [&_[data-slot='slider-track']]:!mask-r-from-99% [&_[data-slot='slider-track']]:!h-1 [&_[data-slot='slider-range']]:!left-[calc(var(--spacing)*1.5+0px))] [&_[data-slot='slider-range']]:!bg-primary/80 [&_[data-slot='slider-range']]:!backdrop-blur-3xl [&_[data-slot='slider-thumb']]:!size-3 [&_[data-slot='slider-thumb']]:!bg-primary [&_[data-slot='slider-thumb']]:!border-none [&_[data-slot='slider-thumb']]:!opacity-0 [&_[data-slot='slider-thumb']]:group-hover/player:!opacity-100 [&_[data-slot='slider-thumb']]:group-active/player:!opacity-100 [&_[data-slot='slider-thumb']]:!transition-opacity [&_[data-slot='slider-thumb']]:!duration-150"
				value={calcSliderValue(MediaPlayerData.progress, MediaPlayerData.duration)}
				min={0}
				max={Math.ceil((MediaPlayerData.duration || 0) * 1000)}
				step={20}
				onValueCommit={(value) => {
					console.log('Progress committed:', value);
				}}
			/>
		</div>

    </div>
    
</section>