<script lang="ts">
	import "./style.css";

	import { invoke } from "@tauri-apps/api/core";

	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { onMount } from 'svelte';
	import { cn } from '$lib/utils.js';

	import { Button } from '$lib/components/ui/button/index.js';
	import * as Select from "$lib/components/ui/select/index.js";

	import {
		Cable,
		Cpu,
		ChevronsLeftRightEllipsis,
		LoaderCircle,
		RotateCcw,
		RouteOff,
	} from '@lucide/svelte/icons';

	import { pyenv_sysinfo, get_nidaq_sysinfo } from "$lib/applications/sysinfo";
	import { type StatusBarDataType, StatusBarData } from ".";
	import { initWsEventListener, closeWsEventListener, wsEventListenerExists } from '@components/media-player/tasks';

	import { goto } from "$app/navigation";

	let {
		class: className
	}: {
		class?: string;
	} = $props();


	pyenv_sysinfo().then((data) => {
		StatusBarData.pySysInfo = data;
		StatusBarData.pywsPid = data.pid;
	});

	function refreshNiDaqInfo() {
		get_nidaq_sysinfo().then((data) => {
			let driver = data.driver;
			if (driver && driver.startsWith("DAQmx")) {
				StatusBarData.niDaqDriverVersion = driver.replace("DAQmx", "");
			} else {
				StatusBarData.niDaqDriverVersion = driver;
			}
	
			StatusBarData.niDaqDevices = data.devices;
			if (data.devices && data.devices.length > 0) {
				StatusBarData.niDaqSelectedDevice = data.devices[0];
			}
		});
	}
	refreshNiDaqInfo();


	let selectedDaqDevice: string | undefined = $derived.by(() => {
		if (StatusBarData.niDaqSelectedDevice) {
			return StatusBarData.niDaqSelectedDevice.name;
		}
		return undefined;
	});
	let deviceSelectorPopupOpened: boolean = $state(false);

	/** 
	 * This function will attempt to check both the Rust-JS, Rust-Python, and Direct WebSocket connections for the process ID and status.
	 */
	async function getPywsPid(): Promise<number | undefined> {
		const pid = await invoke("get_ws_pid");
		const ws = new WebSocket("ws://localhost:21749");

		ws.onopen = () => {
			ws.send(JSON.stringify({ task: "healthcheck" }));
		};

		ws.onmessage = (event) => {
			const result = JSON.parse(event.data);
			if (result.status === "success") {
				ws.close();
				console.log("Healthcheck Status:", result);
				StatusBarData.pywsConnected = true;
				if (!wsEventListenerExists()) {
					initWsEventListener();
				}
			}
			else {
				StatusBarData.pywsConnected = false;
				closeWsEventListener();
			}
		};

		ws.onerror = () => {
			ws.close();
		};

		return pid as number | undefined;
	}
	let pywsPid: Promise<number | undefined> = $derived(getPywsPid());


	onMount(() => {
	});
</script>


<section class={cn(
		'statusbar-override isolate',
		'bg-background/90 pointer-events-auto fixed left-0 bottom-0 z-[99999] border-t border-primary/15',
		className,
		'w-full h-6'
	)}
>
    <div class="w-full h-full flex items-center justify-between gap-2">
		<section data-statusbar-left class="w-fit h-full flex items-center justify-center gap-1.5">
			<div class="ml-1.5 select-none flex items-center justify-center w-fit text-xs text-muted-foreground gap-1">
				<Cpu class="size-3.5" />
				<span class="text-xs text-muted-foreground line-clamp-1 text-ellipsis">
					DAQmx: {StatusBarData.niDaqDriverVersion || 'Unknown'}
				</span>
			</div>

			<Select.Root type="single" bind:value={selectedDaqDevice} bind:open={deviceSelectorPopupOpened}>
				<Select.Trigger class="w-fit text-xs text-muted-foreground gap-1">
					<Cable class="size-3.5" />
					<span class="line-clamp-1 text-ellipsis">
						{selectedDaqDevice || 'None'}
					</span>
				</Select.Trigger>
				<Select.Content class="z-[99999]" side="top" sideOffset={-1} collisionPadding={{top: 42}}>
					<Select.SelectGroup>
						<Select.Label class="py-1 px-1.5">Master NI-DAQmx Device</Select.Label>
						{#if StatusBarData.niDaqDevices && StatusBarData.niDaqDevices.length > 0}
							{#each StatusBarData.niDaqDevices as device}
								<Select.Item value={device.name} class="px-2.5">{device.name}</Select.Item>
							{/each}
						{:else}
							<Select.Item value="none" class="px-2.5 bg-destructive/15" disabled>
								No Devices Connected
								<RouteOff class="inline size-3.5 ml-1" />
							</Select.Item>
						{/if}
					</Select.SelectGroup>
					<Select.Separator />
					<Select.SelectGroup>
						<Select.Label class="py-1 px-1.5">Options</Select.Label>
						<div class="flex flex-col gap-0">
							<Button variant="ghost" class="w-full h-fit py-1.5 justify-start rounded-sm px-2.5 font-normal"
								onclick={() => {
									deviceSelectorPopupOpened = false;
									refreshNiDaqInfo();
								}}
							>
								<div class="w-full flex items-center justify-between gap-4">
									<span>Refresh Devices List</span>
									<RotateCcw class="size-3.5 text-muted-foreground" />
								</div>
							</Button>
							<Button variant="ghost" class="w-full h-fit py-1.5 justify-start rounded-sm px-2.5 font-normal"
								onclick={() => {
									deviceSelectorPopupOpened = false;
									// Go to device config page
									goto(`##`);
								}}
							>
								<span>Configure...</span>
							</Button>
						</div>
					</Select.SelectGroup>
				</Select.Content>
			</Select.Root>
		</section>

		<section data-statusbar-center class="w-fit h-full flex items-center justify-center gap-1.5">
			
		</section>

		<section data-statusbar-right class="w-fit h-full flex items-center justify-center gap-1.5">
			<Button class="px-2 select-none flex items-center justify-center w-fit font-normal text-xs text-muted-foreground gap-1">
				<ChevronsLeftRightEllipsis class="size-4" />
				<span class="text-xs text-muted-foreground line-clamp-1 text-ellipsis flex items-center justify-center gap-1">
					PyWS PID: 
					{#await pywsPid}
						<LoaderCircle class="size-3.5 animate-spin" />
					{:then value}
						{value || "Can't Connect"}
					{:catch error}
						{console.error(error)}
						<span class="text-destructive">Error!</span>
					{/await}
				</span>
			</Button>
		</section>

	</div>
    
</section>