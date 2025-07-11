<script lang="ts">
	let { data } = $props()
	const { scripts } = $derived(data)

	console.log(scripts)
</script>

<main class="flex grow justify-between">
	<ul>scripts</ul>
	<div>content</div>
	<div>menu</div>
</main>
