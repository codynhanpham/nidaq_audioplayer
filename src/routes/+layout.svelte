<script lang="ts" module>
    import { attachConsole } from '@tauri-apps/plugin-log';
    const detach = attachConsole().then((f) => {
        console.log('Global Log Console Attached');
        return f;
    }).catch((error) => {
        console.error('Failed to attach log console:', error);
    });



</script>

<script lang="ts">
    import '../app.css';
    import { ModeWatcher } from "mode-watcher";
    
    import {
		restoreStateCurrent,
		saveWindowState,
		StateFlags
	} from '@tauri-apps/plugin-window-state';
    
	import { getCurrentWindow } from '@tauri-apps/api/window';

    import { TitleBar } from '$lib/components/app-titlebar';
	import type { MenubarData } from '$lib/components/app-titlebar/menubar.svelte';

    import { StatusBar } from '$lib/components/app-statusbar';
    import { MediaPlayer, MediaPlayerData } from '$lib/components/media-player';

    import { onMount, onDestroy } from 'svelte';
    import { cn } from '$lib/utils';

    let { children } = $props();

    restoreStateCurrent(StateFlags.ALL);

    let isDesktop = $state(false);
    let menubarData: MenubarData = $state(null);

    onMount(() => {
        // @ts-ignore
		if (typeof window.__TAURI__ !== 'undefined') {
			isDesktop = true;
		}

        // After the app is loaded, show the app window
		getCurrentWindow().show();

		// Save the window state after the app is loaded
		saveWindowState(StateFlags.ALL);
		// Before reloading the page (with Ctrl R, F5, Ctrl F5, etc.), save the window state
		window.addEventListener('beforeunload', () => {
			saveWindowState(StateFlags.ALL);
		});
    });

    onDestroy(() => {
        saveWindowState(StateFlags.ALL);
    });

</script>

<ModeWatcher />


<div class="flex flex-col h-full w-full gap-0 text-wrap break-all text-ellipsis">
	<TitleBar bind:menubarData />
	
    <div class={cn(
        "mt-8 mb-6 h-full",
        (MediaPlayerData.audioInfo !== null || MediaPlayerData.alwaysShowPlayer) ? "pb-18" : "pb-0"
    )}>
        {@render children?.()}
    </div>

    {#if MediaPlayerData.audioInfo !== null || MediaPlayerData.alwaysShowPlayer}

        <MediaPlayer />
    {/if}

    <StatusBar />
</div>