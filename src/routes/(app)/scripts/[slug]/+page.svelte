<script lang="ts">
	import { mdRenderer } from "$lib/markdown"
	import { DATABASE_URL } from "$lib/supabase"
	import { replaceScriptContent } from "$lib/utils"
	import ScriptHeader from "./ScriptHeader.svelte"
	let { data } = $props()
	const script = $derived(data.script)!
</script>

<ScriptHeader
	title={script.title}
	username={script.protected.username}
	description={script.description}
	stage={script.metadata.stage}
>
	<img
		class="mx-auto h-auto max-h-60 w-full max-w-140 rounded-md xl:mx-0 xl:w-auto xl:max-w-full"
		src={DATABASE_URL + "storage/v1/object/public/imgs/scripts/" + script.id + "/banner.webp"}
		alt="Script banner"
		loading="eager"
	/>
</ScriptHeader>

<div
	class="flex h-full w-full flex-col overflow-y-scroll rounded-md preset-outlined-surface-500 p-8"
>
	{#if !script.published}
		<small class="text-center text-xs text-warning-500">
			This script is not published and not visible for everyone!
		</small>
	{/if}

	<article class="my-4 prose dark:prose-invert">
		{#await data.details then [content, limits]}
			{@html mdRenderer.render(replaceScriptContent(script, content, limits))}
		{/await}
	</article>
</div>
