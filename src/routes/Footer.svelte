<script lang="ts">
	import { page } from "$app/state"
	import Discord from "./Footer/Discord.svelte"
	import GitHub from "./Footer/GitHub.svelte"
	import YouTube from "./Footer/YouTube.svelte"
</script>

<footer
	class="bg-surface-200/30 dark:bg-surface-800/30 flex justify-between text-base font-semibold backdrop-blur-md"
>
	<div class="mx-4 my-4 flex gap-2">
		<GitHub />
		<Discord />
		<YouTube />
	</div>
	<button class="btn preset-filled-primary-500 mx-4 my-4">Run</button>
</footer>
