<script lang="ts">
	import { invoke } from "@tauri-apps/api/core";

	import { Button } from '$lib/components/ui/button/index.js'

	import {
		History,
	} from '@lucide/svelte/icons';

	import { NavigationRoutes } from '$lib/components/app-navigator';
	import { page } from "$app/state";
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
							<route.icon class="size-4 md:size-5 text-foreground/80 group-hover:text-accent-foreground/100" />
						</div>
						<div class="w-full space-y-1">
							<span class="w-full text-foreground/80 group-hover:text-accent-foreground/100 text-wrap line-clamp-1 text-ellipsis">{route.name}</span>
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
			<Button
				variant="ghost"
				class="group h-fit font-normal flex flex-row items-center justify-start gap-3 px-4 py-2 rounded-lg text-left border border-border/90 hover:border-border/100 focus-visible:border-border/100 hover:bg-accent/20 focus-visible:bg-accent/30"
				href="/library/album/12345"
				aria-label="No Recently Played Items"
				title="No Recently Played Items"
				disabled
			>
				<div class="w-fit mx-auto flex items-center justify-center gap-2">
					<History class="size-4 text-foreground/90 group-hover:text-accent-foreground/100" />
					<span class="text-foreground/90 group-hover:text-accent-foreground/100">No recently played items</span>
				</div>
			</Button>

			<!-- {#each STH}
			<Button
				variant="ghost"
				class="group h-fit font-normal flex flex-row items-center justify-start gap-3 px-4 py-2 rounded-lg text-left border border-border/50 hover:border-border/100 focus-visible:border-border/100 hover:bg-accent/20 focus-visible:bg-accent/30"
				href="/library/album/12345"
				aria-label="No Recently Played Items"
				title="No Recently Played Items"
			>
				<div class="w-full flex items-center justify-between gap-2">
					<History class="size-4 text-foreground/90 group-hover:text-accent-foreground/100" />
					<span class="text-foreground/90 group-hover:text-accent-foreground/100">No recently played items</span>
				</div>
			</Button> -->
		</div>
	</section>


	
</main>
