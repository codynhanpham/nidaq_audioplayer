<script lang="ts" module>
    import { attachConsole } from '@tauri-apps/plugin-log';
    const detach = attachConsole().then((f) => {
        console.log('Global Log Console Attached');
        return f;
    }).catch((error) => {
        console.error('Failed to attach log console:', error);
    });


    restoreStateCurrent(StateFlags.ALL);

</script>

<script lang="ts">
    import '../app.css';
    import { ModeWatcher } from "mode-watcher";
    
    import {
		restoreStateCurrent,
		saveWindowState,
		StateFlags
	} from '@tauri-apps/plugin-window-state';
    
	import { register, isRegistered, unregisterAll, type ShortcutEvent } from '@tauri-apps/plugin-global-shortcut';

	import { getCurrentWindow } from '@tauri-apps/api/window';

    import { TitleBar } from '$lib/components/app-titlebar';
	import type { MenubarData } from '$lib/components/app-titlebar/menubar.svelte';

    import { StatusBar } from '$lib/components/app-statusbar';
    import { MediaPlayer, MediaPlayerData, tryPlay } from '$lib/components/media-player';

    import { onMount, onDestroy } from 'svelte';
    import { onNavigate } from '$app/navigation';
    import { cn } from '$lib/utils';

    let { children } = $props();

    let isDesktop = $state(false);
    let menubarData: MenubarData = $state(null);

    
    async function registerCtrlF23() {
        function handleF23(event: ShortcutEvent) {
            if (event.state === 'Pressed') {
                tryPlay();
            }
        }

        if (!(await isRegistered('CmdOrControl+F23'))) {
            await register('CmdOrControl+F23', (event) => {
                handleF23(event);
            });
        }
    }

    onNavigate(() => {
        unregisterAll().then(() => {
            // When navigating to a new page, try re-register the F23 shortcut just to keep the callback fresh (probably some tauri bug?)
            registerCtrlF23();
        });
    });
    

    onMount(() => {
        // @ts-ignore
		if (typeof window.__TAURI__ !== 'undefined') {
			isDesktop = true;
		}

        registerCtrlF23();


        // After the app is loaded, show the app window
		getCurrentWindow().show();

		// Save the window state after the app is loaded
		saveWindowState(StateFlags.ALL);
		// Before reloading the page (with Ctrl R, F5, Ctrl F5, etc.), save the window state
		window.addEventListener('beforeunload', () => {
			saveWindowState(StateFlags.ALL);
		});
    });

    onDestroy(async () => {
        await saveWindowState(StateFlags.ALL);
        await unregisterAll();
    });

</script>

<ModeWatcher />


<div class="fixed top-0 left-0 w-full h-fit z-[999999] isolate">
    <TitleBar bind:menubarData />
</div>

<div class={cn("fixed top-8 left-0 w-full h-full overflow-auto",
        (MediaPlayerData.audioInfo === null && !MediaPlayerData.alwaysShowPlayer) ? "max-h-[calc(100%-(var(--spacing)*8)-(var(--spacing)*6))]" : // the Title + Status Bar height
        (MediaPlayerData.audioInfo?.chapters && MediaPlayerData.audioInfo.chapters.length > 0) ? 'max-h-[calc(100%-(var(--spacing)*8)-(var(--spacing)*6)-(var(--spacing)*21))] sm:max-h-[calc(100%-(var(--spacing)*8)-(var(--spacing)*6)-(var(--spacing)*16.5))]' : 'max-h-[calc(100%-(var(--spacing)*8)-(var(--spacing)*6)-(var(--spacing)*19))] sm:max-h-[calc(100%-(var(--spacing)*8)-(var(--spacing)*6)-(var(--spacing)*16.5))]',
        // "max-h-[calc(100%-(var(--spacing)*8)-(var(--spacing)*6))]"
    )}
>
    <div class={cn(
        "relative h-full",
        // (MediaPlayerData.audioInfo === null && !MediaPlayerData.alwaysShowPlayer) ? "mb-6" : // the Status Bar height
        // (MediaPlayerData.audioInfo?.chapters && MediaPlayerData.audioInfo.chapters.length > 0) ? 'mb-21' : 'mb-19 sm:mb-16.5',
    )}>
        {@render children?.()}
    </div>
</div>

{#if MediaPlayerData.audioInfo !== null || MediaPlayerData.alwaysShowPlayer}
    <div class="fixed bottom-6 left-0 w-full h-fit z-[99999] isolate">
        <div class={cn(
            (MediaPlayerData.audioInfo === null && !MediaPlayerData.alwaysShowPlayer) ? "h-0" :
            ((MediaPlayerData.audioInfo?.chapters && MediaPlayerData.audioInfo.chapters.length > 0) ? 'h-22' : 'h-20'),
            'w-full sm:h-17.5',
            'relative'
        )}>
            <div
                data-ambient-overlay
                class="absolute isolate -z-10 bottom-0 left-0 w-full h-[200%] bg-background/0 backdrop-blur-[54px] pointer-events-none select-none mask-linear-[180deg,transparent_0%,transparent_calc(50%+(var(--spacing))),black_calc(50%+(var(--spacing)))]"
            ></div>
            <MediaPlayer />
        </div>
    </div>
{/if}
    
<StatusBar />