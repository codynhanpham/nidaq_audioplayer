<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";

	import { getCurrentWindow } from '@tauri-apps/api/window';

    import { Button } from '$lib/components/ui/button/index.js'


    import { getCurrentWebview } from "@tauri-apps/api/webview";
    import type { UnlistenFn } from "@tauri-apps/api/event";
    import { onMount } from "svelte";

    import LibrarySelector from './components/library-selector.svelte';

    // onMount(() => {
    //     let dragDropEventUnlistener: UnlistenFn | undefined;
    //     (async () => {
    //         const unlisten = await getCurrentWebview().onDragDropEvent((event) => {
    //             if (event.payload.type === 'over') {
    //             console.log('User hovering', event.payload.position);
    //             } else if (event.payload.type === 'drop') {
    //             console.log('User dropped', event.payload.paths);
    //             } else {
    //             console.log('File drop cancelled');
    //             }
    //         });
    //         dragDropEventUnlistener = unlisten;
    //     })();

    //     return () => {
    //         dragDropEventUnlistener?.();
    //     };
    // });

</script>

<main class="w-full h-full min-h-fit space-y-2 p-2">
    <LibrarySelector />



</main>
