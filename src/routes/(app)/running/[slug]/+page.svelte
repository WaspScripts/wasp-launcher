<script lang="ts">
	import { channelManager } from "$lib/communication.svelte"
	let { data } = $props()

	function transformMessage(msg: string) {
		let style: string | null = null
		if (!msg.startsWith("\0\0")) return { message: msg, style }
		const flag = msg.slice(0, 8)

		if (flag.endsWith("2")) {
			style = "warning"
		} else if (flag.endsWith("4")) {
			style = "error"
		} else if (flag.endsWith("8")) {
			style = "success"
		}

		return { message: msg.slice(8), style }
	}
</script>

{#if channelManager.channels[data.process]}
	{#each channelManager.channels[data.process].logs as msg}
		{@const { message, style } = transformMessage(msg)}
		<p class={style ? `text-${style}-500` : ""}>{message}</p>
	{/each}
{/if}
