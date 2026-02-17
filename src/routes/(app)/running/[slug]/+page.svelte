<script lang="ts">
	import { channelManager } from "$lib/communication.svelte"

	let { data } = $props()

	const STYLES: Record<number, string> = {
		50: "text-yellow-500", //2 Warning
		52: "text-red-500", //4 Error
		56: "text-green-500" //8 Success
	}

	function transformLog(msg: string) {
		if (msg.length < 8 || msg.charCodeAt(0) !== 0 || msg.charCodeAt(1) !== 0) {
			return { text: msg, cls: "" }
		}

		const code = msg.charCodeAt(7)
		return {
			text: msg.slice(8),
			cls: STYLES[code] || ""
		}
	}

	let logs = $derived.by(() => {
		const raw = channelManager.getLogs(data.process)
		const out = new Array(raw.length)
		for (let i = 0; i < raw.length; i++) {
			out[i] = transformLog(raw[i])
		}
		return out
	})
</script>

<div class="font-mono text-sm leading-tight">
	{#each logs as log}
		<p class={log.cls}>{log.text}</p>
	{/each}
</div>
