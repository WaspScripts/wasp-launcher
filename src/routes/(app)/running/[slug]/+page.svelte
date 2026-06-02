<script lang="ts">
	import { channelManager } from "$lib/communication.svelte"
	let { data } = $props()

	let logs = $derived.by(() => {
		const raw = channelManager.getLogs(data.process)
		const out = new Array(raw.length)
		for (let i = 0; i < raw.length; i++) {
			out[i] = raw[i]
		}
		return out
	})
</script>

<div class="font-mono text-sm leading-tight">
	{#each logs as log}
		<span class={`text-[#${log.color}]`}>{log.text}</span>
		{#if log.close}<br />{/if}
	{/each}
</div>
