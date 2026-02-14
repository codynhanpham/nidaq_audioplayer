<script lang="ts" module>
	import { libraryStore, type Library, listLibraryDirs, updateLibraryStore, getScanRecursiveLevel } from "./libraryInfo.svelte";

	export async function addLibraryFolders(paths: string[]) {
		// Check other locations and re-scan them at this time
		const store = await libraryStore;
		const currentLibDir = await listLibraryDirs(store);
		const res = await getScanRecursiveLevel(store);
		paths = Array.from(new Set([...paths, ...currentLibDir]));

		invoke("flex_search_audio_files", {
			paths: paths,
			recursiveLevel: res,
		}).then(async (res: any) => {
			if (res) {
				const library = {
					audioFiles: res.paths,
					libraryStats: res.stats
				} as Library;
				await updateLibraryStore(store, library);		
				await refreshAudioMetadata(true);
			}
		}).catch((err) => {
			console.error("Error adding library folders:", err);
		});
	}

	export async function removeLibraryFolders(path: string) {
		const store = await libraryStore;
		const currentLibDirs = await listLibraryDirs(store);
		const res = await getScanRecursiveLevel(store);
		const newLibDirs = currentLibDirs.filter((dir) => dir !== path);
		invoke("flex_search_audio_files", {
			paths: newLibDirs,
			recursiveLevel: res,
		}).then(async (res: any) => {
			if (res) {
				const library = {
					audioFiles: res.paths,
					libraryStats: res.stats
				} as Library;
				await updateLibraryStore(store, library);		
				await refreshAudioMetadata(true);
			}
		}).catch((err) => {
			console.error("Error removing library folder:", err);
		});
	}

</script>

<script lang="ts">
	import {
		getCurrentWebview,
	} from "@tauri-apps/api/webview";

	import { invoke } from "@tauri-apps/api/core";
	
	import type { PhysicalPosition } from "@tauri-apps/api/dpi";
	import type { UnlistenFn } from "@tauri-apps/api/event";
	import { open } from '@tauri-apps/plugin-dialog';

	import * as Accordion from "$lib/components/ui/accordion/index.js";
	import { onMount, untrack } from "svelte";
	import { type LibraryDirInfo, getLastStoreUpdated } from "./libraryInfo.svelte";
	import DataTable from "./library-location-table/data-table.svelte";
	import { columns } from "./library-location-table/columns";

	import { refreshAudioMetadata } from "../+page.svelte";
    import Button from "@components/ui/button/button.svelte";
    import {
		Plus,
	} from "@lucide/svelte/icons";
    import { LibraryLocationSelector } from "@components/media-player/locationSelectorDisplay.svelte";

	let windowScaleFactor = $state(1);
	let libraryLocationsInfo: Promise<LibraryDirInfo[]> = $derived.by(async () => {
		getLastStoreUpdated();
		const store = await libraryStore;
		const libraryData = await store.get<Library>('library');
		if (!libraryData) return [];
		return libraryData.libraryStats;
	});
	let audioFiles: Promise<string[]> = $derived.by(async () => {
		const store = await libraryStore;
		const libraryData = await store.get<Library>('library');
		if (!libraryData) return [];
		return libraryData.audioFiles;
	});


	function elementContainsPoint(element: HTMLElement, point: PhysicalPosition) {
		const logicalPoints = point.toLogical(windowScaleFactor);
		const pointElement = document.elementFromPoint(logicalPoints.x, logicalPoints.y);
		return element.contains(pointElement);
	}

	function handleDragOver(
		event: {
			type: "over";
			position: PhysicalPosition;
		},
		dropArea: HTMLElement
	) {
		dropArea.classList.add("drag-over-target-hint");
		if (elementContainsPoint(dropArea, event.position)) {
			dropArea.classList.add("active");
		} else {
			dropArea.classList.remove("active");
		}
	}


	

	async function handleDrop(
		event: {
			type: "drop";
			position: PhysicalPosition;
			paths: string[];
		},
		dropArea: HTMLElement
	) {
		dropArea.classList.remove("drag-over-target-hint");
		dropArea.classList.remove("active");

		if (!elementContainsPoint(dropArea, event.position)) {
			return;
		}

		let pathsToSearch = event.paths;

		addLibraryFolders(pathsToSearch);
	}

	function handleResetDragDrop(
		event: {
			type: "enter";
			paths: string[];
			position: PhysicalPosition;
		} | {
			type: "leave";
		},
		dropArea: HTMLElement
	) {
		dropArea.classList.remove("drag-over-target-hint");
		dropArea.classList.remove("active");
	}

	onMount(() => {
		(async () => {
			windowScaleFactor = await getCurrentWebview().window.scaleFactor();
		})();

		const dropArea = document.getElementById("library-locations-container");
		let dragDropEventUnlistener: UnlistenFn | undefined;

		if (dropArea) {
			(async () => {
				const unlisten = await getCurrentWebview().onDragDropEvent((event) => {
				if (event.payload.type === "over") {
					handleDragOver(event.payload, dropArea);
				} else if (event.payload.type === "drop") {
					handleDrop(event.payload, dropArea);
				} else {
					handleResetDragDrop(event.payload, dropArea);
				}
				});
				dragDropEventUnlistener = unlisten;
			})();
		}

		return () => {
		dragDropEventUnlistener?.();
		};
	});


	let accordionValue: string | undefined = $state(undefined);
	$effect(() => {
		if (LibraryLocationSelector.expanded) {
			accordionValue = "library-location";
		} else {
			accordionValue = undefined;
		}
	});
	$effect(() => {
		if (accordionValue === "library-location") {
			untrack(() => {
				LibraryLocationSelector.expanded = true;
			});
		}
		else if (accordionValue === undefined || accordionValue === "") {
			untrack(() => {
				LibraryLocationSelector.expanded = false;
			});
		}
	});


</script>

<section
		id="library-locations-container"
		class="w-full p-3 py-2 bg-muted/40 border border-border/50 rounded-lg drag-over-target"
	>
	<Accordion.Root type="single" class="w-full" bind:value={accordionValue}>
		<Accordion.Item value="library-location">
			<Accordion.Trigger
				class="pt-0 pb-0.5 pr-1 flex items-center justify-between w-full"
			>
				<h2 class="mx-2 text-lg font-semibold">Library Locations</h2>
			</Accordion.Trigger>
			<Accordion.Content
				class="flex flex-col items-center justify-center mt-3 pb-2"
			>
				{#await libraryLocationsInfo}
					<DataTable data={[]} {columns} class="w-full max-w-full" />
				{:then libinfo}
					{@const libinfosorted = libinfo.sort((a, b) => a.dir.localeCompare(b.dir))}
					{#if libinfosorted && libinfosorted.length > 0}
						<DataTable data={libinfosorted} {columns} class="w-full max-w-full" />
					{:else}
					<div class="mt-2 text-center text-sm">
						<p class="text-muted-foreground">No library locations selected</p>
						<p class="text-muted-foreground">Drag and drop here, or click the button below</p>
					</div>
					{/if}
				{:catch}
					<DataTable data={[]} {columns} class="w-full max-w-full" />
				{/await}
				<Button class="mt-4 text-sm" onclick={() => {
					open({
						title: "Select Library Locations",
						directory: true,
						multiple: true,
					}).then((paths) => {
						if (paths && paths.length > 0) {
							addLibraryFolders(paths);
						}
					});
				}}>
					<Plus class="size-4" />
					<span class="mb-[0.5px] mr-1">Add Locations</span>
				</Button>
				
			</Accordion.Content>
		</Accordion.Item>
	</Accordion.Root>
</section>

<style lang="postcss">
	@reference "$src/app.css";
	:global {
		.drag-over-target-hint {
			@apply outline-dashed outline-3 outline-offset-2 outline-ring;
		}
		.drag-over-target-hint.active {
			@apply outline-emerald-700/70 dark:outline-emerald-300/70;
		}
	}
</style>
