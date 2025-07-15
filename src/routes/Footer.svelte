<script lang="ts">
	import { Tooltip } from "@skeletonlabs/skeleton-svelte"
	import { invoke } from "@tauri-apps/api/core"
	import Discord from "./Footer/Discord.svelte"
	import GitHub from "./Footer/GitHub.svelte"
	import YouTube from "./Footer/YouTube.svelte"
	import type { Script } from "$lib/types/collection"

	let { script }: { script?: Script } = $props()

	async function execute() {
		const exe = "simba"
		const args = [script!.id, script!.versions.simba, script!.versions.wasplib]
		const res = await invoke("run_executable", { exe, args })
		console.log(res)
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
