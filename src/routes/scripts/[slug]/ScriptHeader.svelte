<script lang="ts">
	const data = $props()
	let { children } = $derived(data)
	let id: string | undefined = $derived(data.id)
	let title: string | undefined = $derived(data.title)
	let username: string | undefined = $derived(data.username)
	let description: string | undefined = $derived(data.description)
	let hasLink: boolean = $derived(data.hasLink)
</script>

<header class="my-4 flex w-full flex-col justify-between lg:flex-row">
	<div class="mx-auto flex aspect-auto h-full max-h-60 w-auto">
		{@render children()}
	</div>
	<div class="my-auto text-center lg:mx-4">
		<h1 class="my-4 font-bold">
			{title ? title : "Loading..."} by

			{#if hasLink && username}
				<a href="/scripters/{username}"> {username} </a>
			{:else}
				<span> {username ?? "Loading..."} </span>
			{/if}
		</h1>
		<h2 class="my-4">
			{description ?? "Loading..."}
		</h2>
	</div>
</header>
