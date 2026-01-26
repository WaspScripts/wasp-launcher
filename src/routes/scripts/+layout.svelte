<script lang="ts">
	import Navigation from "../Navigation.svelte"
	import Footer from "../Footer.svelte"
	import Sidebar from "../Sidebar.svelte"
	import { page } from "$app/state"

	let { data, children } = $props()
	const scripts = $derived(data.scripts!)
	let search = $state("")

	$inspect(data.script)

	function getStyle(access: boolean, type: string, published: boolean) {
		if (!published) {
			return "text-success-500"
		}

		if (type == "premium") {
			if (access) return "text-primary-500 dark:text-primary-500"
			return "text-warning-500"
		}
	}

	let selected = $state(0)
</script>

<Navigation />
<main class="flex h-full max-h-full justify-between overflow-y-hidden">
	<aside class="flex h-full max-w-96 min-w-64 flex-col border-r border-surface-500 p-2">
		<input
			type="text"
			placeholder="🔍 Search script..."
			class="input text-xs outline-1 outline-surface-300-700 placeholder:text-surface-600-400"
			bind:value={search}
		/>
		<ul class="my-2 h-full overflow-y-scroll">
			{#each scripts as script, idx}
				<li
					class="flex preset-outlined-success-200-800 hover:preset-tonal focus:preset-tonal"
					class:bg-surface-300-700={selected === idx}
					class:border-primary-300-700={selected === idx}
				>
					<a
						href={script.id}
						class="h-full w-full px-2 {getStyle(
							script.access,
							script.metadata.type,
							script.published
						)}"
						onclick={() => (selected = idx)}
					>
						{script.title}
					</a>
				</li>
			{/each}
		</ul>
	</aside>

	<div class="mx-2 flex h-full w-full flex-col gap-y-4 overflow-y-auto">
		{@render children()}
	</div>

	<Sidebar />
</main>
<Footer script={data.script} />
