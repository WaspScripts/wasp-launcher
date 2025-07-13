<script lang="ts">
	import Navigation from "../Navigation.svelte"
	import Footer from "../Footer.svelte"
	import Sidebar from "../Sidebar.svelte"

	let { children } = $props()
</script>

<Navigation />
<main class="flex h-full max-h-full justify-between overflow-y-hidden">
	<div class="flex h-full flex-col overflow-y-auto">
		{@render children()}
	</div>

	<Sidebar />
</main>
<Footer />
