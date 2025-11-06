<script lang="ts">
	import { mardownRenderer } from "$lib/markdown"
	import { DATABASE_URL } from "$lib/supabase"
	import { replaceScriptContent } from "$lib/utils"
	import ScriptHeader from "./ScriptHeader.svelte"
	let { data } = $props()
	const script = $derived(data.script)!

	let limits = $state({
		xp_min: 0,
		xp_max: 0,
		gp_min: 0,
		gp_max: 0
	})

	async function getLimits() {
		const { data: limitsData, error: err } = await data.supabase
			.schema("stats")
			.from("limits")
			.select("xp_min, xp_max, gp_min, gp_max")
			.eq("id", script.id)
			.single()
		if (err) {
			console.error(err)
			return
		}
		limits = limitsData
	}

	getLimits()

	let content = $derived(replaceScriptContent(script, limits))
</script>

<ScriptHeader
	id={script.id}
	title={script.title}
	username={script.protected.username}
	description={script.description}
>
	<img
		class="rounded-md"
		src={DATABASE_URL + "storage/v1/object/public/imgs/scripts/" + script.id + "/banner.jpg"}
		alt="Script banner"
		loading="lazy"
	/>
</ScriptHeader>

{#if !script.published}
	<span class="text-center text-warning-500">
		This script is not published and not visible for everyone!
	</span>
{/if}

<div class="mx-2 mt-4 mb-2 flex h-full overflow-y-scroll rounded-md preset-outlined-surface-500">
	<article class="m-4 prose dark:prose-invert">
		{@html mardownRenderer.render(content)}
	</article>
</div>
