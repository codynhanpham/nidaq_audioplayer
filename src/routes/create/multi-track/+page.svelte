<script lang="ts" module>

</script>


<script lang="ts">
	import {
		getCurrentWebview,
	} from "@tauri-apps/api/webview";
    import { invoke } from "@tauri-apps/api/core";
	
	import type { PhysicalPosition } from "@tauri-apps/api/dpi";
	import type { UnlistenFn } from "@tauri-apps/api/event";
	import { open } from '@tauri-apps/plugin-dialog';

	import { Button } from '$lib/components/ui/button/index.js'
    import {
    } from "@lucide/svelte/icons";

	import { onMount } from "svelte";
    



    let windowScaleFactor = $state(1);
        
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

		let pathsToSearch = event.paths[0];
        const playlist = await invoke("parse_playlist", { path: pathsToSearch });
        console.log(playlist);
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

		const dropArea = document.getElementById("multitrack-file-dropzone");
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
</script>

<main class="w-full h-full min-h-fit space-y-2 p-4 md:p-8 flex flex-col items-center justify-center">
    <div
		id="multitrack-file-dropzone"
		class="w-full p-3 bg-muted/40 border border-border/50 rounded-lg drag-over-target flex flex-1 items-center justify-center"
    >
        <h2 class="text-lg text-center font-semibold mb-2">Drop your .yml file here</h2>
    </div>
</main>

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
