<script lang="ts">
	import { Tooltip } from "@skeletonlabs/skeleton-svelte"
	import { invoke } from "@tauri-apps/api/core"

	let { icon, label, exe, args } = $props()

	async function execute() {
		const res = await invoke("run_executable", { exe, args })
		console.log(res)
	}
	let openState = $state(false)
</script>

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
		<button class="btn preset-filled-surface-500 [&>*]:pointer-events-none" onclick={execute}>
			{icon}
		</button>
	{/snippet}
	{#snippet content()}{label}{/snippet}
</Tooltip>
