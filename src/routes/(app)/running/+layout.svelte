<script lang="ts">
	import { running } from "$lib/communication.js"
	import { SearchIcon } from "@lucide/svelte"

	let { data, children } = $props()
	let search = $state("")

	$inspect(data.script)

	let selected = $state(0)
</script>

<aside
	class="flex h-full max-w-96 min-w-44 flex-col gap-2 border-r border-surface-500 p-2 text-sm lg:min-w-64"
>
	<div class="input-group h-9 grid-cols-[auto_1fr_auto]">
		<div class="ig-cell preset-tonal px-2">
			<SearchIcon size={16} />
		</div>
		<input
			type="text"
			placeholder="Search script..."
			class="input ig-input outline-1 outline-surface-300-700 placeholder:text-surface-600-400"
			bind:value={search}
		/>
	</div>

	<ul class="h-full w-full overflow-y-scroll">
		{#each running as script, idx}
			<li
				class="flex preset-outlined-success-200-800 hover:preset-tonal focus:preset-tonal"
				class:bg-surface-300-700={selected === idx}
				class:border-primary-300-700={selected === idx}
			>
				<a
					href={"/running/" + script}
					class="my-2 flex h-full w-full justify-between px-2"
					onclick={() => (selected = idx)}
				>
					{script}
				</a>
			</li>
		{/each}
	</ul>
</aside>

<main class="flex h-full w-full overflow-y-auto">
	{@render children()}
</main>
