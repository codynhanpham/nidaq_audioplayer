<script lang="ts">
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { onMount } from 'svelte';
	import { cn } from '$lib/utils.js';

	import { Button } from '$lib/components/ui/button/index.js';

	import IconMinus from '@lucide/svelte/icons/minus';

	let {
		class: className
	}: {
		class?: string;
	} = $props();



	onMount(async () => {
	});
</script>


<section class={cn(
		'bg-background/90 pointer-events-auto fixed left-0 bottom-0 z-[99999] border-t border-primary/15',
		className,
		'w-full h-6'
	)}
>
    
    
</section>