<script lang="ts">
	import ScriptStage from "$lib/components/ScriptStage.svelte"
	import { SearchIcon } from "@lucide/svelte"

	let { data, children } = $props()
	let search = $state("")

	$effect.pre(() => {
		data.script?.id
		search = ""
	})

	const scripts = $derived.by(() => {
		const query = search.trim().toLowerCase()
		if (!query) return data.scripts
		return data.scripts.filter((script) => script.title.toLowerCase().includes(query))
	})

	function getStyle(access: boolean, type: string, published: boolean) {
		if (!published) {
			return "text-success-500"
		}

		if (type == "premium") {
			if (access) return "text-primary-500 dark:text-primary-500"
			return "text-warning-500"
		}
	}
</script>

<aside
	class="flex h-full max-w-96 min-w-44 flex-col gap-2 border-r border-surface-500 py-2 pl-2 text-sm lg:min-w-64"
>
	<div class="mr-2 input-group grid-cols-[auto_1fr_auto] overflow-visible text-scaling">
		<div class="ig-cell px-1 md:px-2">
			<SearchIcon size={16} />
		</div>
		<input
			type="text"
			placeholder="Search script..."
			class="ig-input h-full placeholder:text-surface-600-400"
			bind:value={search}
		/>
	</div>

	<ul class="h-full overflow-y-scroll pr-2">
		{#each scripts as script (script.id)}
			<li
				class="flex preset-outlined-surface-200-800 hover:preset-tonal focus:preset-tonal"
				class:bg-surface-300-700={data.script?.id === script.id}
				class:border-primary-300-700={data.script?.id === script.id}
			>
				<a
					href={script.id}
					class="w-full px-1 py-2 md:px-2 {getStyle(
						script.access,
						script.metadata.type,
						script.published
					)} flex justify-between"
				>
					{script.title}
					<ScriptStage stage={script.metadata.stage} size={12} styles={"px-1 text-xs lg:text-sm"} />
				</a>
			</li>
		{/each}
	</ul>
</aside>

<div class="mx-2 flex h-full w-full flex-col gap-y-4 overflow-y-auto">
	{@render children()}
</div>
