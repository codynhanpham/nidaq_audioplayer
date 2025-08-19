<script lang="ts" module>

</script>


<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";

	import { Button } from '$lib/components/ui/button/index.js'
    import {
        ListMusic,
        Merge
    } from "@lucide/svelte/icons";

    

</script>

<main class="w-full h-full min-h-fit space-y-2 p-4 md:p-8 flex flex-col items-center justify-center">
    
	<section class="w-full max-w-prose p-3 bg-muted/40 border border-border rounded-lg">
        <h2 class="mx-2 text-xl font-semibold">New Project</h2>
        <div class="mt-4 mb-2 grid gap-x-4 gap-y-4 grid-cols-[repeat(auto-fit,minmax(8rem,1fr))] md:grid-cols-[repeat(auto-fit,minmax(9rem,1fr))] my-3 mx-2">
            <Button class="w-full" role="link" href="/create/multi-track">
                <ListMusic class="size-4.5 mr-2" />
                Multi-Track
            </Button>
            <Button variant="secondary" class="w-full" role="link" href="/create/join-gapless">
                <Merge class="size-4.5 mr-2 rotate-180" />
                Join Gapless
            </Button>
        </div>
    </section>
    
	<!-- <section class="w-full max-w-prose p-3 border border-border rounded-lg">
        <h2 class="mx-2 text-xl font-semibold">Open...</h2>
        <div class="mt-4 mb-2 w-full">
            <p class="w-full text-center text-muted-foreground">No Recent Projects</p>
        </div>
    </section> -->


</main>
