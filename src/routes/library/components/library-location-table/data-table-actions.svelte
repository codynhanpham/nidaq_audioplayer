<script lang="ts">
    import EllipsisIcon from "@lucide/svelte/icons/ellipsis";
    import { Button } from "$lib/components/ui/button/index.js";
    import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
    import { openPath } from '@tauri-apps/plugin-opener';
    import { removeLibraryFolders } from "../library-selector.svelte";
    import { ExternalLink, Trash, Trash2 } from "@lucide/svelte";

    let { path }: { path: string } = $props();
</script>

<DropdownMenu.Root>
    <DropdownMenu.Trigger class="max-w-fit">
        {#snippet child({ props })}
            <Button
                {...props}
                variant="ghost"
                size="icon"
                class="relative w-8 h-6 p-0 sm:mr-2 md:mr-4 lg:mr-6"
            >
                <span class="sr-only">Open option menu</span>
                <EllipsisIcon />
            </Button>
        {/snippet}
    </DropdownMenu.Trigger>
    <DropdownMenu.Content align="end" alignOffset={-4} sideOffset={0} collisionPadding={{top: 42, right: 2}}>
        <DropdownMenu.Group>
            <DropdownMenu.Label class="text-muted-foreground">Actions</DropdownMenu.Label>
            <DropdownMenu.Item
                onclick={() => openPath(path)}
            >
                <ExternalLink class="text-foreground" />
                Open in File Explorer
            </DropdownMenu.Item>
            
            <DropdownMenu.Separator />

            <DropdownMenu.Item variant="destructive"
                onclick={() => removeLibraryFolders(path)}
            >
                <Trash2 />
                Remove from Library
            </DropdownMenu.Item>
        </DropdownMenu.Group>
    </DropdownMenu.Content>
</DropdownMenu.Root>
