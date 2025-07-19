<script lang="ts">
	import { Tooltip } from "@skeletonlabs/skeleton-svelte"
	import { invoke } from "@tauri-apps/api/core"
	import Discord from "./Footer/Discord.svelte"
	import GitHub from "./Footer/GitHub.svelte"
	import YouTube from "./Footer/YouTube.svelte"
	import type { Script } from "$lib/types/collection"
	import { page } from "$app/state"
	import type { SupabaseClient } from "@supabase/supabase-js"
	import type { Database } from "$lib/types/supabase"

	let { script }: { script?: Script } = $props()
	const supabase: SupabaseClient<Database> = $derived(page.data.supabase)

	async function saveBlobToFile(blob: Blob, filePath: string) {
		const arrayBuffer = await blob.arrayBuffer()
		const bytes = Array.from(new Uint8Array(arrayBuffer))

		await invoke("save_blob", {
			filePath,
			data: bytes
		})
	}

	async function execute() {
		const { data, error: err } = await supabase.storage
			.from("scripts")
			.download(script!.id + "/" + script!.protected.revision + "/script.simba")
		if (err) {
			console.error(err)
			return
		}

		await saveBlobToFile(data, "Simba/Scripts/" + script!.id + ".simba")

		const exe = "simba"
		const args = [script!.id, script!.versions.simba, script!.versions.wasplib]
		await invoke("run_executable", { exe, args })
	}
	let openState = $state(false)
</script>

<footer
	class="bg-surface-200/30 dark:bg-surface-800/30 sticky bottom-0 flex justify-between text-base font-semibold backdrop-blur-md"
>
	<div class="mx-4 my-4 flex gap-2">
		<Tooltip
			open={openState}
			onOpenChange={(e) => (openState = e.open)}
			positioning={{ placement: "top" }}
			triggerBase="underline"
			contentBase="card preset-filled p-4"
			openDelay={1000}
			arrow
		>
			{#snippet trigger()}
				<GitHub />
			{/snippet}
			{#snippet content()}Source code{/snippet}
		</Tooltip>

		<Tooltip
			open={openState}
			onOpenChange={(e) => (openState = e.open)}
			positioning={{ placement: "top" }}
			triggerBase="underline"
			contentBase="card preset-filled p-4"
			openDelay={1000}
			arrow
		>
			{#snippet trigger()}
				<Discord />
			{/snippet}
			{#snippet content()}Join the Discord community!{/snippet}
		</Tooltip>

		<Tooltip
			open={openState}
			onOpenChange={(e) => (openState = e.open)}
			positioning={{ placement: "top" }}
			triggerBase="underline"
			contentBase="card preset-filled p-4"
			openDelay={1000}
			arrow
		>
			{#snippet trigger()}
				<YouTube />
			{/snippet}
			{#snippet content()}YouTube channel{/snippet}
		</Tooltip>
	</div>

	{#if script}
		<Tooltip
			open={openState}
			onOpenChange={(e) => (openState = e.open)}
			positioning={{ placement: "top" }}
			triggerBase="underline"
			contentBase="card preset-filled p-4"
			openDelay={1000}
			arrow
		>
			{#snippet trigger()}
				<button class="btn preset-filled-primary-500 mx-4 my-4" onclick={execute}>Open</button>
			{/snippet}
			{#snippet content()}Open in Simba{/snippet}
		</Tooltip>
	{/if}
</footer>
