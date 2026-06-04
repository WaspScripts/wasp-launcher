<script lang="ts">
	import { channelManager } from "$lib/communication.svelte"
	import { onMount, tick } from "svelte"
	const { data } = $props()
	let container: HTMLDivElement
	let parent: HTMLDivElement
	const logs = $derived([...channelManager.getLogs(data.process)])

	function scrollDown() {
		if (!parent) return
		parent.scrollTop = parent.scrollHeight
	}

	$effect(() => {
		logs
		tick().then(() => scrollDown())
	})

	onMount(() => (parent = container.parentElement as HTMLDivElement))
</script>

<div bind:this={container} class="font-mono text-sm leading-tight">
	{#each logs as log}
		<span style="color:#{log.color}">{log.text}</span>
		{#if log.close}<br />{/if}
	{/each}
</div>
