<script lang="ts">
	import "./style.css";

	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { onMount } from 'svelte';
	import { cn } from '$lib/utils.js';

	import { Button } from '$lib/components/ui/button/index.js';
	import * as Select from "$lib/components/ui/select/index.js";

	import {
		Cable,
		Cpu,
	} from '@lucide/svelte/icons';

	import { pyenv_sysinfo, get_nidaq_sysinfo } from "$lib/applications/sysinfo";

	import { type StatusBarDataType, StatusBarData } from ".";
  import SelectGroup from "@components/ui/select/select-group.svelte";

	let {
		class: className
	}: {
		class?: string;
	} = $props();


	pyenv_sysinfo().then((data) => {
		StatusBarData.pySysInfo = data;
		StatusBarData.pywsPid = data.pid;
	});

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

	let selectedDaqDevice: string | undefined = $derived.by(() => {
		if (StatusBarData.niDaqSelectedDevice) {
			return StatusBarData.niDaqSelectedDevice.name;
		}
		return undefined;
	});


	onMount(async () => {
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
			<div class="ml-2 select-none flex items-center justify-center w-fit text-xs text-muted-foreground gap-1">
				<Cpu class="size-3.5" />
				<span class="text-xs text-muted-foreground">
					DAQmx: {StatusBarData.niDaqDriverVersion || 'Unknown'}
				</span>
			</div>

			<Select.Root type="single" bind:value={selectedDaqDevice}>
				<Select.Trigger class="w-fit text-xs text-muted-foreground gap-1">
					<Cable class="size-3.5" />
					<span>
						{selectedDaqDevice || 'None'}
					</span>
				</Select.Trigger>
				<Select.Content class="z-[99999]" side="top" sideOffset={-1}>
					<SelectGroup>
						<Select.Label class="pt-0.5 pb-1.5 px-1.5">Master NI-DAQmx Device</Select.Label>
						{#if StatusBarData.niDaqDevices && StatusBarData.niDaqDevices.length > 0}
							{#each StatusBarData.niDaqDevices as device}
								{console.log(device)}
								<Select.Item value={device.name} class="px-2.5">{device.name}</Select.Item>
							{/each}
						{/if}
					</SelectGroup>
				</Select.Content>
			</Select.Root>
		</section>

		<section data-statusbar-center class="w-fit h-full flex items-center justify-center gap-1.5">
			
		</section>

		<section data-statusbar-right class="w-fit h-full flex items-center justify-center gap-1.5">
			
		</section>

	</div>
    
</section>