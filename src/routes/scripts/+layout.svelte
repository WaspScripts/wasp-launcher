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
	<aside class="flex h-full max-w-96 min-w-64 flex-col border-r-1 border-surface-500 p-2">
		<input
			type="text"
			placeholder="🔍 Search script..."
			class="input text-xs outline-1 outline-surface-300-700 placeholder:text-surface-600-400"
			bind:value={search}
		/>
		<ul class="my-2 h-full overflow-y-scroll">
			{#each scripts as script}
				<li class="flex preset-outlined-success-200-800 hover:preset-tonal">
					<a
						href={script.id}
						class="h-full w-full px-2 {script.metadata.type == 'premium'
							? script.access
								? 'text-primary-500'
								: 'text-warning-500'
							: ''}">{script.title}</a
					>
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
