<script>
	import ExecuteButton from "$lib/components/ExecuteButton.svelte"
	import LinkButton from "$lib/components/LinkButton.svelte"
</script>

<div class="border-surface-500 flex h-full flex-col justify-end gap-1 border-l-1 px-1">
	<ExecuteButton icon="🦁" label="Open Simba" exe="simba" args={[]} />
	<ExecuteButton icon="💻" label="RuneLite" exe="runelite" args={[]} />
	<ExecuteButton icon="🚀" label="OSClient" exe="osclient" args={[]} />
	<LinkButton icon="⚙️" label="Settings" href="/settings" />
</div>
