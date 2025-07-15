<script lang="ts">
	import Navigation from "../Navigation.svelte"
	import Footer from "../Footer.svelte"
	import Sidebar from "../Sidebar.svelte"

	let { data, children } = $props()
	const scripts = $derived(data.scripts!)

	let search = $state("")

	$inspect(data.script)
</script>

<Navigation />
<main class="flex max-h-full justify-between overflow-y-hidden">
	<aside class="border-surface-500 flex h-full max-w-96 min-w-64 flex-col border-r-1 p-2">
		<input
			type="text"
			placeholder="🔍 Search script ..."
			class="input placeholder:text-surface-400-600 text-xs"
			bind:value={search}
		/>
		<ul class="my-2 h-full overflow-y-scroll">
			{#each scripts as script}
				<li class="hover:text-primary-500 preset-outlined-success-200-800 flex">
					<a href={script.id} class="h-full w-full px-2">{script.title}</a>
				</li>
			{/each}
		</ul>
	</aside>

	<div class="flex h-full flex-col overflow-y-auto">
		{@render children()}
	</div>

	<Sidebar />
</main>
<Footer script={data.script} />
