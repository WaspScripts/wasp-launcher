<script lang="ts">
	let { children } = $props()
</script>

<div class="flex h-full w-full flex-col overflow-y-auto">
	{@render children()}
</div>
