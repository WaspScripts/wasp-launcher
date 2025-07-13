<script lang="ts">
	import { Tooltip } from "@skeletonlabs/skeleton-svelte"

	let { icon, label, href } = $props()

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
		<a {href} class="btn preset-filled-surface-500 [&>*]:pointer-events-none">
			{icon}
		</a>
	{/snippet}
	{#snippet content()}{label}{/snippet}
</Tooltip>
