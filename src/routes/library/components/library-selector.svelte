<script lang="ts">
	import {
		getCurrentWebview,
		type DragDropEvent,
	} from "@tauri-apps/api/webview";
	
	import type { PhysicalPosition } from "@tauri-apps/api/dpi";
	import type { UnlistenFn } from "@tauri-apps/api/event";

	import * as Accordion from "$lib/components/ui/accordion/index.js";
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

	function handleDrop(
		event: {
			type: "drop";
			position: PhysicalPosition;
			paths: string[];
		},
		dropArea: HTMLElement
	) {
		dropArea.classList.remove("drag-over-target-hint");
		dropArea.classList.remove("active");
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
			console.log(dropArea);
			const unlisten = await getCurrentWebview().onDragDropEvent((event) => {
			if (event.payload.type === "over") {
				console.log(event.payload.position);
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

<section
		id="library-locations-container"
		class="w-full p-3 bg-muted/40 border border-border/50 rounded-lg drag-over-target"
	>
	<Accordion.Root type="single" class="w-full" value={undefined}>
		<Accordion.Item value="library-location">
		<Accordion.Trigger
			class="pt-0 pb-1 pr-1 flex items-center justify-between w-full"
		>
			<h2 class="mx-2 text-lg font-semibold">Library Locations</h2>
		</Accordion.Trigger>
		<Accordion.Content
			class="flex flex-col items-center justify-center"
		></Accordion.Content>
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
