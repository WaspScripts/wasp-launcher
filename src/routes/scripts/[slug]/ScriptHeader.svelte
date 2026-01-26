<script lang="ts">
	import type { TScriptStages } from "$lib/types/collection"
	import { scriptStages } from "$lib/utils"
	import { Portal, Tooltip } from "@skeletonlabs/skeleton-svelte"

	const {
		children,
		title,
		username,
		description,
		stage
	}: {
		children: any
		title: string
		username: string
		description: string
		stage: TScriptStages
	} = $props()
</script>

<header class="flex w-full flex-col justify-evenly gap-4 xl:flex-row">
	{@render children()}

	<div class="mx-auto my-auto w-96 text-center">
		<h1 class="my-4 h5 font-bold">
			{title ? title : "Loading..."} by
			<span> {username ?? "Loading..."} </span>
		</h1>
		<h2 class="my-4">
			{description ?? "Loading..."}
		</h2>

		{#if stage !== "stable"}
			<Tooltip positioning={{ placement: "bottom" }}>
				<Tooltip.Trigger
					class="mx-auto my-2 w-fit cursor-default rounded-md preset-outlined-surface-600-400 preset-filled-surface-500 px-2 py-1 hover:preset-tonal"
				>
					{scriptStages[stage].icon + scriptStages[stage].name}
				</Tooltip.Trigger>
				<Portal>
					<Tooltip.Positioner>
						<Tooltip.Content class="card preset-filled-surface-950-50 p-2">
							<span>
								This script is in <b>{scriptStages[stage].name}</b> and might not be stable!
							</span>
							<Tooltip.Arrow
								class="[--arrow-background:var(--color-surface-950-50)] [--arrow-size:--spacing(2)]"
							>
								<Tooltip.ArrowTip />
							</Tooltip.Arrow>
						</Tooltip.Content>
					</Tooltip.Positioner>
				</Portal>
			</Tooltip>
		{/if}
	</div>
</header>
