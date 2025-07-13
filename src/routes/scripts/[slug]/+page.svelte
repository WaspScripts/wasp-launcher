<script lang="ts">
	import { PUBLIC_SUPABASE_URL } from "$env/static/public"
	import { mardownRenderer } from "$lib/markdown"
	import { replaceScriptContent } from "$lib/utils"
	import ScriptHeader from "./ScriptHeader.svelte"
	let { data } = $props()
	const script = $derived(data.script)
</script>

<ScriptHeader
	id={script.id}
	title={script.title}
	username={script.protected.username}
	description={script.description}
>
	<img
		class="rounded-md"
		src={PUBLIC_SUPABASE_URL +
			"/storage/v1/object/public/imgs/scripts/" +
			script.id +
			"/banner.jpg"}
		alt="Script banner"
		loading="lazy"
	/>
</ScriptHeader>

<div
	class="preset-outlined-surface-500 mx-2 mt-4 mb-2 flex h-full min-h-72 overflow-y-scroll rounded-md"
>
	<article class="prose dark:prose-invert m-4">
		{@html mardownRenderer.render(replaceScriptContent(script))}
	</article>
</div>
